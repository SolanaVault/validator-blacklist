use anyhow::{Context, Result};
use anchor_client::solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

/// Represents a parsed validator entry from `solana validators get` output
#[derive(Debug, Clone)]
pub struct ValidatorEntry {
    pub identity: Pubkey,
    pub has_warning: bool,
}

/// Parse the output from `solana validators get` command
/// 
/// The format looks like:
/// ```
///    Identity                                      Vote Account                            Commission  Last Vote        Root Slot     Skip Rate  Credit
///   3iQqh65Gby53aaYUF8ocoiEyhBs4aoe7BTYYWvy1c9dF  AEtdq4CwtuktCEUWLLpRTNPBZs6tr7BBqxkHJ1DjAttR    5%  379088558 (-10)  379088527 (-10)    -     234800
/// ⚠️2XK1YYuLwPCMZSbmfedmso1vmkqrX63M2srNApvAntvw  ENjAU1VZvBTAMCwg9ZayfxLaRQEPExcR2ujH7VdeBkDh  100%  378433337        378433306          -
/// ```
pub fn parse_validator_list(input: &str) -> Result<Vec<ValidatorEntry>> {
    let mut entries = Vec::new();
    let mut header_skipped = false;

    for line in input.lines() {
        let trimmed = line.trim();

        // Skip empty lines
        if trimmed.is_empty() {
            continue;
        }

        // Skip header line - it contains "Identity" and "Vote Account"
        if !header_skipped && (trimmed.contains("Identity") && trimmed.contains("Vote Account")) {
            header_skipped = true;
            continue;
        }

        // Skip the header if not yet skipped (in case it appears without those exact words)
        if !header_skipped && (trimmed.starts_with("Identity") || trimmed.starts_with("--")) {
            header_skipped = true;
            continue;
        }

        // Parse the line
        if let Ok(entry) = parse_validator_line(trimmed) {
            entries.push(entry);
        }
    }

    Ok(entries)
}

/// Parse a single line from the validator list
fn parse_validator_line(line: &str) -> Result<ValidatorEntry> {
    // Check if line starts with warning emoji
    let has_warning = line.starts_with("⚠️");

    // Remove the emoji prefix if present
    let line_content = if has_warning {
        // Remove the ⚠️ emoji (which is 3 bytes in UTF-8)
        &line[3..].trim_start()
    } else {
        line.trim_start()
    };

    // Split by whitespace and get the first non-empty part
    let parts: Vec<&str> = line_content.split_whitespace().collect();

    if parts.is_empty() {
        anyhow::bail!("No columns found in line: {}", line);
    }

    // The first column is the identity address
    let identity_str = parts[0];

    // Parse the identity as a Pubkey
    let identity = Pubkey::from_str(identity_str)
        .context(format!("Failed to parse identity address: {}", identity_str))?;

    Ok(ValidatorEntry {
        identity,
        has_warning,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_validator_line_without_warning() {
        let line = "3iQqh65Gby53aaYUF8ocoiEyhBs4aoe7BTYYWvy1c9dF  AEtdq4CwtuktCEUWLLpRTNPBZs6tr7BBqxkHJ1DjAttR    5%  379088558 (-10)  379088527 (-10)    -     234800";
        let result = parse_validator_line(line).unwrap();
        assert_eq!(result.identity.to_string(), "3iQqh65Gby53aaYUF8ocoiEyhBs4aoe7BTYYWvy1c9dF");
        assert!(!result.has_warning);
    }

    #[test]
    fn test_parse_validator_line_with_warning() {
        let line = "⚠️2XK1YYuLwPCMZSbmfedmso1vmkqrX63M2srNApvAntvw  ENjAU1VZvBTAMCwg9ZayfxLaRQEPExcR2ujH7VdeBkDh  100%  378433337        378433306          -";
        let result = parse_validator_line(line).unwrap();
        assert_eq!(result.identity.to_string(), "2XK1YYuLwPCMZSbmfedmso1vmkqrX63M2srNApvAntvw");
        assert!(result.has_warning);
    }

    #[test]
    fn test_parse_validator_list() {
        let input = r#"   Identity                                      Vote Account                            Commission  Last Vote        Root Slot     Skip Rate  Credit
  3iQqh65Gby53aaYUF8ocoiEyhBs4aoe7BTYYWvy1c9dF  AEtdq4CwtuktCEUWLLpRTNPBZs6tr7BBqxkHJ1DjAttR    5%  379088558 (-10)  379088527 (-10)    -     234800
  BULKzD8ZgbYV6taZjXYkdSytcutscMGTFFi2MDHViKdc  BULKqDiPxzBJPB27nHNRjz1iKKBU252FtntLjhf7EjQZ  100%  379088558 (-10)  379088527 (-10)    -     235171
⚠️2XK1YYuLwPCMZSbmfedmso1vmkqrX63M2srNApvAntvw  ENjAU1VZvBTAMCwg9ZayfxLaRQEPExcR2ujH7VdeBkDh  100%  378433337        378433306          -"#;

        let result = parse_validator_list(input).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].identity.to_string(), "3iQqh65Gby53aaYUF8ocoiEyhBs4aoe7BTYYWvy1c9dF");
        assert!(!result[0].has_warning);
        assert_eq!(result[2].identity.to_string(), "2XK1YYuLwPCMZSbmfedmso1vmkqrX63M2srNApvAntvw");
        assert!(result[2].has_warning);
    }
}

