use futures::prelude::*;
use sonor::Speaker;
use std::{time::Duration, env::Args, collections::HashMap};

const MUSIC_SERVICE: &str = "urn:upnp-org:serviceId:MusicServices";

#[tokio::main]
async fn main() -> Result<(), sonor::Error> {

    let mut args = std::env::args();
    let argv0 = args.next().expect("Cannot get argv[0]");

    // Handle args
    let subcommand = args.next().unwrap_or_else(|| panic!("wrong # args, try {argv0} subcommand"));

    match subcommand.as_ref() {
        "list" => {
            do_list().await?;
        }
        "join" => {
            do_join(args).await?;
        }
        _ => {
            panic!("Unknown subcommand, try {argv0} list|join")
        }
    }
    
    Ok(())
}

async fn do_join(mut args: Args) -> Result<(), sonor::Error> {
    
    if args.len() < 2 {
        panic!("Supply at least two speaker zones to join");
    }

    let speakers = get_speakers().await?;
    
    let group_master_name = args.next().unwrap();
    let group_master_zone = speakers.get(&group_master_name).unwrap_or_else(|| panic!("Unknown zone: {}", group_master_name));
    let group_master = group_master_zone.get(0).unwrap();

    // join the master to all the others
    for arg in args {
        let ref spkr: Speaker = speakers.get(&arg).unwrap_or_else(|| panic!("Unknown speaker {}", arg))[0];
        join(group_master, spkr).await?;

        println!("Joined {arg} to {group_master_name}");
    }

    Ok(())
}

async fn join(master: &Speaker, member: &Speaker) -> Result<(), sonor::Error> {
    let uri = format!("x-rincon:{}", master.uuid().await?);

    // Cannot use .join(&str) - which finds the Boost
    member.set_transport_uri(&uri, "").await?;

    // eprintln!("{} <- {}", master.name().await?, member.name().await?);

    Ok(())
}

async fn do_list() -> Result<(), sonor::Error> {

    for (zone, speakers) in get_speakers().await?.into_iter() {

        let mut uids = vec![];
        for spkr in &speakers {
            uids.push(spkr.uuid().await?);
        }

        println!("{}: {} - {:?}", zone, speakers.len(), uids);
    }

    Ok(())
}

/// Retun true for only those devices that are speakers.
/// Which carry a service with service_id = "urn:upnp-org:serviceId:MusicServices"
fn is_speaker(device: &Speaker) -> bool {
    device.device()
        .services_iter()
        .any(|s| s.service_id() == MUSIC_SERVICE)

}

/// Return a list of speakers, that implement MUSIC_SERVICE.
async fn get_speakers() -> Result<HashMap<String, Vec<Speaker>>, sonor::Error> {
    let mut devices = sonor::discover(Duration::from_secs(5)).await?;   

    let mut result = HashMap::new();
    while let Some(speaker) = devices.try_next().await? {
        if is_speaker(&speaker) {
            result.entry(speaker.name().await?)
                .or_insert_with(Vec::new)
                .push(speaker);
        }
    }

    Ok(result)
}

