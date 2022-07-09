use c2pa::{
    assertions::{User, Action, Actions, c2pa_action},
    Manifest, Error,
    get_signer_from_files, 
    ManifestStore
};

use std::path::PathBuf;
use std::fs::File;

// output file
const DESTINATION_PATH: &str = "/tmp/claims/test-c2pa-trump-trial-with-claims.jpeg";

// source image
const SOURCE_PATH: &str = "./jan6.jpeg";

fn write_assertion() -> Result<(), Error> {
    // init sample manifest
    let mut manifest = Manifest::new("test_app".to_owned());
 
    /**
     * add a simple action to the manifest
     * this isn't a great action since it's probably a value already
     * included in the library but it's a test val that we can write
     * and read back later on
     */
    let actions = Actions::new().add_action(
        Action::new(c2pa_action::CREATED)
        .set_parameter("at", "07102022".to_owned())?
    );

    // add the assertion to the manifest
    let result = manifest.add_assertion(&actions);

    if !result.is_ok() {
        println!("invalid assertion: {:?}", result.err());
    }

    /**
     * now that the manifest is made we need to generate a new image with the signed manifest
     */


    let source = PathBuf::from(SOURCE_PATH);
    let path = PathBuf::from(DESTINATION_PATH);


    /**
     * pull in the certificate and private key
     * I had difficulty generating these (since creating a self-signed cert is not straightforward), so I
     * pulled the pub/pem files from the repo examples
     */
    let signcert_path  = "/tmp/keys/c2pa.pub";
    let pkey_path = "/tmp/keys/c2pa.pem";

    // note that to use the `get_signer_from_files, the `file_io` feature needs to be enabled
    let signer_result = get_signer_from_files(signcert_path, pkey_path, "ps256", None);


    // this is some extra logging to debug
    match signer_result {
        Err(err) => println!("got an issue {:?}", err),
        Ok(result) => {
            // embed the manifest in the destination image file via the signer created above
            let other_result = manifest.embed(&source, &path, &*result);
            if !other_result.is_ok() {
                println!("whoopsie {:?}", other_result.err());
            }
        }
    }



    Ok(())
}

fn read_source() -> Result<(), Error> {
    // retrieve the manifest embedded in the destination image asset
    let store = ManifestStore::from_file(SOURCE_PATH)?;

    // print out the entire store since it is nicely formatted here
    println!("{}", store);

    if let Some(manifest) = store.get_active() {
        /**
         * fetch a bucket of actions (and organize by label?)
         * this bit was unclear to me.  not entirely sure what "label"
         * is here, but it allows us to get out what we put in above.
         */
        let actions: Actions = manifest.find_assertion(Actions::LABEL)?;
        for action in actions.actions {
            // this should print the "created" we wrote to the manifest above
            println!("got an action: {}\n", action.action());
        }
    }
    Ok(())
}

fn read_dest() -> Result<(), Error> {
        let store = ManifestStore::from_file(DESTINATION_PATH)?;
        println!("printing store: {}", store);

        if let Some(manifest) = store.get_active() {
            let actions_result: Result<Actions, Error> = manifest.find_assertion(Actions::LABEL);
            match actions_result {
                Ok(actions) => {
                    for action in actions.actions {
                        println!("got an action: {}\n", action.action());
                        println!("action stuff: {:?}", action);
                    }
                },
                Err(err) => {
                    println!("had an issue getting the assertion with actions label: {:?}", err);
                }
            }

        } 
    Ok(())
}

fn main() -> Result<(), Error> {
    println!("writing assertion to new file");
    let write_result = write_assertion();
    if write_result.is_err() {
        println!("encountered an error writing the assertion: {:?}", write_result.err());
    }

    println!("reading original source for manifest / claims");
    let result = read_source();

    if result.is_err() {
        println!("had an issue reading a manifest from the first source: {:?}", result.err());
    }

    println!("reading new generated file with manifest");
    let result = read_dest();
    if result.is_err() {
        println!("had an issue reading generated file: {:?}", result.err());
    } else {
        println!("destination ok");
    }
    Ok(())
}