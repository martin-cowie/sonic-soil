use futures::prelude::*;
use sonor::Speaker;
use std::{time::Duration, env::Args, collections::BTreeMap};

const MUSIC_SERVICE: &str = "urn:upnp-org:serviceId:MusicServices";

#[tokio::main]
async fn main() -> Result<(), sonor::Error> {

    let mut args = std::env::args();
    let argv0 = args.next().expect("Cannot get argv[0]");

    // Handle args
    let subcommand = args.next().unwrap_or_else(|| {
        eprintln!("Usage: {argv0} <list|join>");
        std::process::exit(1);
    });

    match subcommand.as_ref() {
        "list" => {
            do_list().await?;
        }
        "join" => {
            do_join(args).await?;
        }
        _ => {
            eprintln!("Unknown subcommand '{}', try: {argv0} list|join", subcommand);
            std::process::exit(1);
        }
    }
    
    Ok(())
}

async fn do_join(mut args: Args) -> Result<(), sonor::Error> {
    
    if args.len() < 2 {
        eprintln!("Supply at least two speaker zones to join");
        std::process::exit(1);
    }

    let speakers = get_speakers().await?;

    let group_master_name = args.next().unwrap();

    // join each member to the master by room name
    for arg in args {
        let spkr = &speakers.get(&arg).unwrap_or_else(|| {
            eprintln!("Unknown speaker: {}", arg);
            std::process::exit(1);
        })[0];

        match spkr.join(&group_master_name).await {
            Ok(true) => println!("Joined {arg} to {group_master_name}"),
            Ok(false) => eprintln!("Could not find zone '{group_master_name}' on the network"),
            Err(e) => eprintln!("Failed to join {arg} to {group_master_name}: {e:#?}"),
        }
    }

    Ok(())
}

async fn do_list() -> Result<(), sonor::Error> {
    for (zone, speakers) in get_speakers().await?.into_iter() {
        let uids: Vec<_> = futures::future::try_join_all(
            speakers.iter().map(|s| s.uuid())
        ).await?;

        let ips: Vec<_> = speakers.iter()
            .map(|s| s.device().url().host().unwrap_or("unknown").to_string())
            .collect();

        println!("{}: UIDs={:?} IPs={:?}", zone, uids, ips);
    }

    Ok(())
}

/// Return true for only those devices that are speakers.
/// Which carry a service with service_id = "urn:upnp-org:serviceId:MusicServices"
fn is_speaker(device: &Speaker) -> bool {
    device.device()
        .services_iter()
        .any(|s| s.service_id() == MUSIC_SERVICE)

}

/// Return a list of speakers, that implement MUSIC_SERVICE.
async fn get_speakers() -> Result<BTreeMap<String, Vec<Speaker>>, sonor::Error> {
    let mut devices = sonor::discover(Duration::from_secs(5)).await?;

    let mut result = BTreeMap::new();
    while let Some(speaker) = devices.try_next().await? {
        if is_speaker(&speaker) {
            result.entry(speaker.name().await?)
                .or_insert_with(Vec::new)
                .push(speaker);
        }
    }

    Ok(result)
}

