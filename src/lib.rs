#[cfg(feature = "regex")]
pub extern crate regex;

pub mod rt {
    #[repr(align(1))]
    pub struct DenseDFABytes8<const N: usize>(pub [u8; N]);

    #[repr(align(2))]
    pub struct DenseDFABytes16<const N: usize>(pub [u8; N]);

    #[repr(align(4))]
    pub struct DenseDFABytes32<const N: usize>(pub [u8; N]);
}

#[cfg(feature = "build")]
pub fn write_regex<W: std::io::Write>(
    name: &str,
    pattern: &str,
    mut out: W,
) -> Result<(), Box<dyn std::error::Error>> {
    use regex_automata::RegexBuilder;

    fn comment_pattern<W: std::io::Write>(mut w: W, pattern: &str) -> Result<(), std::io::Error> {
        writeln!(w, "/// ```\n")?;
        for line in pattern.lines() {
            writeln!(w, "/// {}", line.trim_end())?;
        }
        writeln!(w, "/// ```\n")
    }

    #[cfg(feature = "regex")]
    fn write_jit<W: std::io::Write>(
        name: &str,
        pattern: &str,
        mut out: W,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("cargo:warning=Falling back to regex for: {pattern}");

        comment_pattern(&mut out, pattern)?;

        write!(
            out,
            r##"pub static {name}: once_cell::sync::Lazy<regex_build::regex::Regex> = once_cell::sync::Lazy::new(|| regex_build::regex::Regex::new({pattern:?}).unwrap());"##
        )?;

        Ok(())
    }

    let mut anchored = false;
    let mut stripped = pattern;
    if let Some(re) = stripped.strip_prefix('^') {
        stripped = re;
        anchored = true;
    }

    let res = RegexBuilder::new()
        .minimize(true)
        .ignore_whitespace(true)
        .unicode(true)
        .anchored(anchored)
        .build_with_size::<u16>(stripped);

    let re = match res {
        Ok(re) => re,
        #[cfg(feature = "regex")]
        Err(_) => return write_jit(name, pattern, out),
        #[cfg(not(feature = "regex"))]
        Err(e) => return Err(e.into()),
    };

    let mut size = 16;
    let mut forward = re.forward().to_bytes_native_endian()?;
    let mut reverse = re.reverse().to_bytes_native_endian()?;

    // try to shrink to u8s if possible
    if let (Ok(f), Ok(r)) = (re.forward().to_u8(), re.reverse().to_u8()) {
        size = 8;
        forward = f.to_bytes_native_endian()?;
        reverse = r.to_bytes_native_endian()?;
    }

    #[cfg(feature = "regex")]
    {
        let total_bytes = forward.len() + reverse.len();

        if total_bytes > (200 * 1024) {
            return write_jit(name, pattern, out);
        }
    }

    comment_pattern(&mut out, pattern)?;

    write!(
        out,
        r#"pub static {name}: once_cell::sync::Lazy<Regex<DenseDFA<&'static [u{size}], u{size}>>> = once_cell::sync::Lazy::new(|| unsafe {{
            Regex::from_dfas(
                DenseDFA::from_bytes(&regex_build::rt::DenseDFABytes{size}({forward:?}).0),
                DenseDFA::from_bytes(&regex_build::rt::DenseDFABytes{size}({reverse:?}).0)
            )
        }});"#
    )?;

    Ok(())
}
