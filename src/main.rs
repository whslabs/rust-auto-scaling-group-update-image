use aws_config::meta::region::RegionProviderChain;
use aws_sdk_autoscaling::{Client, Error};
use clap::Parser;

#[derive(Parser)]
struct Cli {
    #[arg(required = true)]
    name: Option<String>,

    #[arg(long, value_name = "NEW_AMI_ID", required = true)]
    new_ami_id: Option<String>,

    #[arg(long, value_name = "NEW_LAUNCH_CONFIGURATION_NAME", required = true)]
    new_launch_configuration_name: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");

    let config = aws_config::from_env().region(region_provider).load().await;

    let client = Client::new(&config);

    let r = client
        .describe_auto_scaling_groups()
        .auto_scaling_group_names(cli.name.as_ref().unwrap())
        .send()
        .await?;

    println!("{:?}", r);

    let r = client
        .describe_launch_configurations()
        .launch_configuration_names(
            r.auto_scaling_groups
                .unwrap()
                .first()
                .unwrap()
                .launch_configuration_name
                .as_ref()
                .unwrap(),
        )
        .send()
        .await?;

    println!("{:?}", r);

    let launch_configuration = r
        .launch_configurations
        .unwrap()
        .clone()
        .first()
        .unwrap()
        .clone();

    println!("{:?}", launch_configuration);

    let r = client
        .create_launch_configuration()
        .set_image_id(cli.new_ami_id)
        .set_instance_type(launch_configuration.instance_type)
        .set_key_name(launch_configuration.key_name)
        .set_launch_configuration_name(cli.new_launch_configuration_name.clone())
        .set_security_groups(launch_configuration.security_groups)
        .send()
        .await?;

    println!("{:?}", r);

    let r = client
        .update_auto_scaling_group()
        .set_auto_scaling_group_name(cli.name)
        .set_launch_configuration_name(cli.new_launch_configuration_name)
        .send()
        .await?;

    println!("{:?}", r);

    Ok(())
}
