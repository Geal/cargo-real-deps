use cargo::{
    core::{
        compiler::{CompileKind, RustcTargetData},
        resolver::{CliFeatures, ForceAllTargets, HasDevUnits},
        FeatureValue, PackageIdSpec, Workspace,
    },
    ops,
    util::{
        command_prelude::{App, Arg},
        config::Config,
        errors::CargoResult,
        interning::InternedString,
    },
};
use std::{collections::BTreeSet, path::Path, rc::Rc};

fn main() -> CargoResult<()> {
    let matches = App::new("cargo-real-deps")
        .arg(
            Arg::with_name("path")
                .short("p")
                .long("path")
                .required(true)
                .takes_value(true)
                .help("path to Cargo.toml"),
        )
        .arg(
            Arg::with_name("all-features")
                .long("all-features")
                .help("activate all features"),
        )
        .arg(
            Arg::with_name("no-default-features")
                .long("no-default-features")
                .help("deactivate default features"),
        )
        .arg(
            Arg::with_name("features")
                .long("features")
                .takes_value(true)
                .value_delimiter(",")
                .help("activates some features"),
        )
        .get_matches();

    let path = Path::new(matches.value_of("path").unwrap())
        .canonicalize()
        .unwrap();
    let all_features = matches.is_present("all-features");
    let uses_default_features = !matches.is_present("no-default-features");
    let features = Rc::new(
        matches
            .values_of("features")
            .map(|v| {
                v.map(InternedString::new)
                    .map(FeatureValue::new)
                    .collect::<BTreeSet<_>>()
            })
            .unwrap_or_default(),
    );

    let config = Config::default()?;
    let ws = Workspace::new(&path, &config)?;
    let specs = vec![PackageIdSpec::from_package_id(
        ws.current().unwrap().package_id(),
    )];

    let targets = &[CompileKind::Host][..];
    let resolve = ops::resolve_ws_with_opts(
        &ws,
        &RustcTargetData::new(&ws, targets)?,
        targets,
        &CliFeatures {
            features,
            all_features,
            uses_default_features,
        },
        &specs,
        HasDevUnits::No,
        ForceAllTargets::No,
    )?
    .targeted_resolve;

    let package_ids = resolve.sort();
    /*
    println!("metadata: {:?}", resolve.metadata());
    let packige = ws.current()?;
    println!("current package: {:?}", packige);
    println!("current id: {:?}", packige.package_id());
    //println!("summary: {:?}", packige.summary());
    //println!("targets: {:#?}", packige.targets());
    let members = ws.members().collect::<Vec<_>>();
    println!("workspace members: {:?}", members);
    */

    for id in &package_ids {
        println!("{} {} {:?}", id.name(), id.version(), resolve.features(*id));
    }

    Ok(())
}
