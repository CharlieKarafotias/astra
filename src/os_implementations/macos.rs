use std::error::Error;
use std::process::Command;

// --- OS specific code ---
pub(crate) fn get_screen_resolution() -> Result<(u32, u32), MacOSError> {
    let output = Command::new("system_profiler")
        .arg("SPDisplaysDataType")
        .arg("-detailLevel")
        .arg("mini")
        .output()
        .map_err(|_| MacOSError::SystemProfilerError)?;

    let (width, height) = parse_output(&String::from_utf8_lossy(&output.stdout))?;
    Ok((width, height))
}

// --- OS specific code ---

// --- Helper functions ---
fn parse_output(output: &str) -> Result<(u32, u32), MacOSError> {
    // find line with Main Display: Yes
    let main_display_idx = output.lines()
        .position(|x| x.contains("Main Display: Yes"))
        .ok_or(MacOSError::MainDisplayNotFound)?;
    println!("Main display line: {}", output.lines().nth(main_display_idx).unwrap());

    // count spaces preceding it until new line
    let main_display_line = output.lines().nth(main_display_idx)
        .ok_or(MacOSError::MainDisplayNotFound)?;
    let trimmed_line = main_display_line.trim_start_matches(' ');
    let num_spaces = main_display_line.len() - trimmed_line.len();
    println!("Number of spaces: {}", num_spaces);

    // grab all lines with that many spaces preceding them
    let mut properties: Vec<&str> = vec![];
    // check up
    let mut i = main_display_idx - 1;
    while i > 0 {
        let line = output.lines().nth(i).unwrap();
        let trimmed_line = line.trim_start_matches(' ');
        let curr_line_spaces = line.len() - trimmed_line.len();
        if curr_line_spaces != num_spaces {
            break;
        }
        properties.push(line);
        i -= 1;
    }
    // check down
    let mut i = main_display_idx + 1;
    while i < output.lines().count() {
        let line = output.lines().nth(i).unwrap();
        let trimmed_line = line.trim_start_matches(' ');
        let curr_line_spaces = line.len() - trimmed_line.len();
        if curr_line_spaces != num_spaces {
            break;
        }
        properties.push(line);
        i += 1;
    }

    // find line with Resolution: and grab next 2 numbers
    Ok(properties.iter()
        .find(|x| x.contains("Resolution:"))
        .ok_or(MacOSError::ResolutionNotFound)
        .and_then(|x| {
            let mut resolution_vals = x
                .split(" x ")
                .map(|x| {
                    let num: String = x.chars().filter(|c| c.is_digit(10)).collect();
                    num.parse::<u32>().map_err(|_| MacOSError::ResolutionNotFound)
                })
                .collect::<Result<Vec<u32>, MacOSError>>()?;

            if resolution_vals.len() != 2 {
                return Err(MacOSError::ResolutionNotFound);
            }
            Ok((resolution_vals[0], resolution_vals[1]))
        }))?
}

// --- Helper functions ---

// --- Errors ---
#[derive(Debug, PartialEq)]
pub enum MacOSError {
    MainDisplayNotFound,
    ResolutionNotFound,
    SystemProfilerError,
}

impl std::fmt::Display for MacOSError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MacOSError::MainDisplayNotFound => write!(f, "Unable to determine main display"),
            MacOSError::ResolutionNotFound => write!(f, "Unable to determine resolution of main display"),
            MacOSError::SystemProfilerError => write!(f, "Encountered error running system_profiler"),
        }
    }
}

impl Error for MacOSError {}
// --- Errors ---

// --- Tests ---

#[cfg(test)]
mod macos {
    #[test]
    fn it_parses_valid_output() {
        let output = r#"
            Graphics/Displays:

                Apple M1:

                  Chipset Model: Apple M1
                  Type: GPU
                  Bus: Built-In
                  Total Number of Cores: 8
                  Vendor: Apple (0x106b)
                  Metal Support: Metal 3
                  Displays:
                    Color LCD:
                      Display Type: Built-In Retina LCD
                      Resolution: 2560 x 1600 Retina
                      Main Display: Yes
                      Mirror: Off
                      Online: Yes
                      Automatically Adjust Brightness: Yes
                      Connection Type: Internal
                    LG HDR WFHD:
                      Resolution: 2560 x 1080 (UW-UXGA - Ultra Wide - Ultra Extended Graphics Array)
                      UI Looks like: 2560 x 1080 @ 75.00Hz
                      Mirror: Off
                      Online: Yes
                      Rotation: Supported
            "#;

        let (width, height) = super::parse_output(output).unwrap();
        assert_eq!(width, 2560);
        assert_eq!(height, 1600);
    }

    #[test]
    fn it_fails_to_parse_invalid_output() {
        let output = "";
        let res = super::parse_output(output);
        assert_eq!(res.err().unwrap(), super::MacOSError::MainDisplayNotFound);
    }

    #[test]
    fn it_fails_when_no_main_display() {
        let output = "Main Display: No";
        let res = super::parse_output(output);
        assert_eq!(res.err().unwrap(), super::MacOSError::MainDisplayNotFound);
    }

    #[test]
    fn it_fails_when_no_resolution_found() {
        let output = "Main Display: Yes";
        let res = super::parse_output(output);
        assert_eq!(res.err().unwrap(), super::MacOSError::ResolutionNotFound);
    }
}

// --- Tests ---