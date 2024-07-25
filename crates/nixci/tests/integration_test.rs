//! Integration tests for nixci
//!
//! These tests are disabled by default (because they won't build in Nix
//! sandbox), and must be manually enabled using the feature flag.
#[cfg(feature = "integration_test")]
mod integration_test {
    use std::{
        path::{Path, PathBuf},
        process::Command,
    };

    use assert_cmd::cargo::CommandCargoExt;
    use clap::Parser;
    use nixci::{self, cli, nix::nix_store::StorePath};
    use regex::Regex;

    #[ctor::ctor]
    fn init() {
        nixci::logging::setup_logging(true);
    }

    #[tokio::test]
    /// Run `nixci build` and check if the stdout consists of only /nix/store/* paths
    async fn nixci_build_output() -> anyhow::Result<()> {
        let mut cmd = Command::cargo_bin("nixci")?;
        cmd.arg("build")
            .arg("github:srid/haskell-multi-nix/c85563721c388629fa9e538a1d97274861bc8321");

        let output = cmd.output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let lines = stdout.lines();

        for line in lines {
            assert!(
                line.starts_with("/nix/store/"),
                "Unexpected line in stdout: {}",
                line
            );
        }

        Ok(())
    }

    #[tokio::test]
    /// A simple test, without config
    async fn test_haskell_multi_nix() -> anyhow::Result<()> {
        let args = cli::CliArgs::parse_from([
            "nixci",
            "-v",
            "build",
            "github:srid/haskell-multi-nix/c85563721c388629fa9e538a1d97274861bc8321",
        ]);
        let outs = nixci::nixci(args).await?;
        let drv_outs: Vec<PathBuf> = outs
            .into_iter()
            .filter_map(|drv_result| {
                if let StorePath::Other(drv_out) = drv_result {
                    Some(drv_out)
                } else {
                    None
                }
            })
            .collect();
        let expected = vec![
            "/nix/store/3x2kpymc1qmd05da20wnmdyam38jkl7s-ghc-shell-for-packages-0",
            "/nix/store/dzhf0i3wi69568m5nvyckck8bbs9yrfd-foo-0.1.0.0",
            "/nix/store/hsj8mwn9vzlyaxzmwyf111scisnjhlkb-bar-0.1.0.0",
            "/nix/store/hsj8mwn9vzlyaxzmwyf111scisnjhlkb-bar-0.1.0.0/bin/bar",
        ]
        .into_iter()
        .map(|s| PathBuf::from(s.to_string()))
        .collect::<Vec<_>>();
        assert_same_drvs(drv_outs, expected);
        Ok(())
    }

    #[tokio::test]
    async fn test_haskell_multi_nix_all_dependencies() -> anyhow::Result<()> {
        let args = cli::CliArgs::parse_from([
            "nixci",
            "-v",
            "build",
            "--print-all-dependencies",
            "github:srid/haskell-multi-nix/c85563721c388629fa9e538a1d97274861bc8321",
        ]);
        let outs = nixci::nixci(args).await?;
        // Since the number of dependencies is huge, we just check for the presence of system-independent
        // source of the `foo` sub-package in `haskell-multi-nix`.
        let expected = StorePath::Other(PathBuf::from(
            "/nix/store/bpybsny4gd5jnw0lvk5khpq7md6nwg5f-source-foo",
        ));
        assert!(outs.contains(&expected));
        Ok(())
    }

    #[tokio::test]
    /// A test, with config
    async fn test_services_flake() -> anyhow::Result<()> {
        let args = cli::CliArgs::parse_from([
            "nixci",
            "-v",
            "build",
            // TODO: Change after merging https://github.com/juspay/services-flake/pull/51
            "github:juspay/services-flake/3d764f19d0a121915447641fe49a9b8d02777ff8",
        ]);
        let outs = nixci::nixci(args).await?;
        let drv_outs: Vec<PathBuf> = outs
            .into_iter()
            .filter_map(|drv_result| {
                if let StorePath::Other(drv_out) = drv_result {
                    Some(drv_out)
                } else {
                    None
                }
            })
            .collect();
        let expected = vec![
            "/nix/store/1vlflyqyjnpa9089dgryrhpkypj9zg76-elasticsearch",
            "/nix/store/20dz7z6pbzpx6sg61lf2sihj286zs3i2-postgres-test",
            "/nix/store/4h6zn33lk2zpb7ch4ljd7ik6fk4cqdyi-nix-shell",
            "/nix/store/6r5y4d7bmsqf0dk522rdkjd1q6ffiz2p-treefmt-check",
            "/nix/store/87mhdmfs479rccyh89ss04ylj7rmbbyl-redis",
            "/nix/store/8aq4awsrggaflv7lg5bp2qkmx52isqfk-redis-test",
            "/nix/store/8xm6ccnbxkm2vapk084gmr89x8bvkh7i-redis-cluster-test",
            "/nix/store/h604nx70yi7ca0zapwls6nlhy7n396lq-zookeeper-test",
            "/nix/store/ibp162hp3wb3zz3hkwlfbq45ivmymj80-redis-cluster",
            "/nix/store/ilx0c8gvyqviyn4wy0xsc8l9lmxq2g66-postgres",
            "/nix/store/mhlzq02nmqn3wn4f2vhyq8sgf44siqkv-zookeeper",
            "/nix/store/pahcafwnm9hj58wzlgfldm9k2g5794qr-nix-shell",
            "/nix/store/pcds2jxvqr9ahyyff50r3qv5y5b944xz-default-test",
            "/nix/store/pczvahjnzp01qzk1z4ixgialbmyxq3f0-apache-kafka-test",
            "/nix/store/pl6m18fsz16kd59bg4myhvkfv04syb65-elasticsearch-test",
            "/nix/store/wcvfpxciyv4v3w35fxc9axbvdv0lv13d-apache-kafka",
            "/nix/store/y3xlr9fnsq43j175b3f69k5s7qw0gh8p-default",
        ]
        .into_iter()
        .map(|s| PathBuf::from(s.to_string()))
        .collect::<Vec<_>>();
        assert_same_drvs(drv_outs, expected);
        Ok(())
    }

    pub fn assert_same_drvs(drvs1: Vec<PathBuf>, drvs2: Vec<PathBuf>) {
        assert_eq!(drvs1.len(), drvs2.len());
        let mut drv1 = drvs1
            .into_iter()
            .map(|d| without_hash(&d))
            .collect::<Vec<_>>();
        let mut drv2 = drvs2
            .into_iter()
            .map(|d| without_hash(&d))
            .collect::<Vec<_>>();
        drv1.sort();
        drv2.sort();
        assert_eq!(drv1, drv2);
    }

    pub fn without_hash(out_path: &Path) -> String {
        let re = Regex::new(r".+\-(.+)").unwrap();
        let captures = re.captures(out_path.to_str().unwrap()).unwrap();
        captures.get(1).unwrap().as_str().to_string()
    }
}
