use ptree::TreeBuilder;
use regex::Regex;

#[derive(Debug, PartialEq, Eq)]
enum Flag {
    StartNewFile,
    ReturnToFile,
    SystemHeader,
    WrappedInExternC,
}
#[derive(Debug)]
struct UnrecognizedFlagError;
impl std::str::FromStr for Flag {
    type Err = UnrecognizedFlagError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Flag::*;
        Ok(match s {
            "1" => StartNewFile,
            "2" => ReturnToFile,
            "3" => SystemHeader,
            "4" => WrappedInExternC,
            _ => return Err(UnrecognizedFlagError),
        })
    }
}

fn line_number_and_path(prefixes: &[String], n: &str, p: &str) -> (usize, String) {
    let line_number: usize = n.parse().expect("Failed to parse line number");
    let path = prefixes
        .iter()
        .find(|pre| p.starts_with(*pre))
        .and_then(|pre| p.strip_prefix(pre))
        .unwrap_or(p)
        .to_owned();
    (line_number, path)
}

fn main() {
    let line_regex =
        Regex::new(r#"# (\d+) "([^"]*)"((?: \d)*)"#).expect("Failed to process regular expression");
    let flags_regex = Regex::new(r"\d").expect("Failed to process regular expression");
    let prefixes: Vec<_> = std::env::args().skip(1).collect();

    let mut tree = TreeBuilder::new("root".to_string());
    let mut head = &mut tree;
    let mut current_line = 0;

    let events = std::io::stdin()
        .lines()
        .map_while(Result::ok)
        .filter(|v| v.starts_with('#'))
        // extract field
        .filter_map(|s| {
            let captures = line_regex.captures(&s)?;
            let mut grp = captures.iter().skip(1).flatten().map(|v| v.as_str());
            let n = grp.next()?;
            let p = grp.next()?;
            let (line_number, path) = line_number_and_path(&prefixes, n, p);
            let flags = grp
                .next()
                .map(|f| {
                    flags_regex
                        .find_iter(f)
                        .map(|m| m.as_str().parse().expect("Failed to parse flag"))
                        .collect()
                })
                .unwrap_or_else(Vec::new);
            Some((line_number, path, flags))
        });

    for (line, file, flags) in events {
        match flags.as_slice() {
            s if s.contains(&Flag::StartNewFile) => {
                head.begin_child(format!("{current_line} {file}"));
            }
            s if s.contains(&Flag::ReturnToFile) => {
                head = head.end_child();
                current_line = line;
            }
            _ => current_line = line,
        }
    }
    let tree = head.build();
    ptree::output::print_tree(&tree).expect("Failed to print the include tree");
}
