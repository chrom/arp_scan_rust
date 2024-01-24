use std::str::FromStr;
use clap::ArgMatches;
use ipnetwork::{Ipv4Network};

#[derive(Debug)]
pub enum OutputFormat {
    Plain,
    Json,
    Yaml,
    Csv
}

#[derive(Debug)]
pub enum ProfileType {
    Default,
    Fast,
    Stealth,
    Chaos
}

#[derive(Debug)]
pub struct CliOptions {
    pub profile: ProfileType,
    pub output: OutputFormat,
    pub network: Ipv4Network,
}

impl CliOptions {
    pub fn new(matches: &ArgMatches) -> Result<CliOptions, String> {
        let profile = Self::get_profile(matches)?;
        let output = Self::get_output(matches)?;
        let network = Self::get_network(matches)?;

        Ok(CliOptions {
            profile,
            output,
            network
        })
    }

    fn get_profile(matches: &ArgMatches) -> Result<ProfileType, String> {
        let profile = matches.get_one::<String>("profile");
        if profile.is_none() {
            return Err("Profile not provided".to_string());
        }

        let result = match profile.unwrap().as_str() {
            "default" | "d" => ProfileType::Default,
            "fast" | "f" => ProfileType::Fast,
            "stealth" | "s" => ProfileType::Stealth,
            "chaos" | "c" => ProfileType::Chaos,
            _ => unreachable!("Expected correct profile name {{default|fast|stealth|chaos}}")
        };
        Ok(result)
    }

    fn get_output(matches: &ArgMatches) -> Result<OutputFormat, String> {
        let output = matches.get_one::<String>("output");
        if output.is_none() {
            return Err("Profile not provided".to_string());
        }


        let result = match output.unwrap().as_str() {
            "plain" | "p" => OutputFormat::Plain,
            "json" | "j" => OutputFormat::Json,
            "yaml" | "y" => OutputFormat::Yaml,
            "csv" | "c" => OutputFormat::Csv,
            _ => unreachable!("Expected correct output name {{plain|json|yaml|csv}}")
        };
        Ok(result)
    }

    fn get_network(matches: &ArgMatches)
        -> Result<Ipv4Network, String>
    {
        let network = matches.get_one::<String>("network")
            .ok_or("Network not provided")?
            .as_str();
        let result = Ipv4Network::from_str(network)
            .map_err(|e| format!("Failed to parse IP address: {}", e.to_string()))?;
        Ok(result)
    }

}