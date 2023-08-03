# Benchmarking Ansible Automation

This folder houses all the [Ansible](https://www.ansible.com/) roles to automate the configuration of your local
environment and to deploy the necessary DynamoDB and DAX components to AWS. AWS Deployments leverage 
[AWS CDK](https://aws.amazon.com/cdk/) to automate the provisioning of AWS resources. For more information, 
navigate to the [CDK directory](../cdk/README.md).

To just see how to run different plays and their corresponding commands without knowing how it all works together,
skip down to the [Plays](#plays) section below.

Note that if no `ssh_key_name` is provided, the default value is `$USER-dax-pair`

## Prerequisites
* You must be logged into the AWS CLI prior to running the CDK. Ensure you're logged into your target AWS account by running
  `aws sts get-caller-identity`.
* Install pip (Assuming python3 is already installed): `sudo apt-get install python3-pip`
* Install the most recent version of Ansible and jmespath from pip: `pip3 install --user ansible jmespath`
* Export the local bin path: `export PATH=~/.local/bin:$PATH`
* Install curl (`sudo apt-get install curl`)
* Install the required Ansible dependencies using Ansible Galaxy (`ansible-galaxy install -r requirements.yml`)

## Initializing the Stack
To initialize the stack (including the local Elastic Stack), run the `deploy_benchmarker.yml` playbook with the `init` tag:
```shell
ansible-playbook -i inventories/local \
--tags init \
--ask-become-pass \
deploy_benchmarker.yml
```

## Deploying the Stack
To deploy the entire benchmarking stack all at once, local and AWS, use the following command:
```shell
ansible-playbook -i inventories/local \
-e vpc_id={{ vpc_id_to_deploy_into }} \
deploy_benchmarker.yml
```

The same prerequisites apply to the CDK with the necessary environment or CDK parameters as is defined in the 
[CDK Parameters](../cdk/README.md#cdk-arguments) section of the CDK README. Ansible will only resolve the following variables
for you; all other variables must be supplied by the user a runtime:

* `localIp`
* `awsAccount`

## Running the benchmarkers
To run the benchmarkers, run the following command:
```shell
ansible-playbook -i inventories/local \
-e dax_endpoint={{ the_dax_endpoint_uri }} \
run_benchmarkers.yml
```

### Ansible Command Breakdown
Let's analyze how an ansible command is formed:

```shell
ansible-playbook -i inventories/local \
-e vpc_id={{ vpc_id_to_deploy_into }} \
--ask-become-pass \
deploy_benchmarker.yml
```

`ansible-playbook` is the program that runs our playbook, `deploy_benchmarker.yml`. [Playbooks](https://docs.ansible.com/ansible/latest/user_guide/playbooks_intro.html)
are the main "blueprints" of automation tasks that Ansible uses.

`-i inventories/local` tells Ansible that we want to use the hosts and variables associated
with the `local` environment. So later in the playbook and
[roles](https://docs.ansible.com/ansible/latest/user_guide/playbooks_reuse_roles.html), when we're
using variables and hosts, we're pulling the corresponding values for this environment. More
information about inventories in Ansible can be found
[here](https://docs.ansible.com/ansible/2.3/intro_inventory.html). Inventories would be a good place
to start learning about Ansible if you're confused by what's happening in this module.

[This](./inventories/local/host_vars/localhost.yml) is where you'd put variables to persist between runs of this application. 
By default, they are only provided for you if you follow the steps in the main repository script.

`-e vpc_id={{ vpc_id_to_deploy_into }}` is setting an extra variable for the playbook to use (fun fact: `-e` is an alias
for `--extra-vars`). This variable is not defined by default in your [local host vars](./inventories/local/host_vars/localhost.yml) because
we don't know what VPC you want to deploy the stack into. If you're running this using the main TUI script in the root
of this repo, then this is handled graphically for you. This will be set on the first run of the CDK deployment, so you do not have to specify
the `vpc_id` between subsequent runs. Otherwise, if you wish to change the VPC ID for any reason (including prior to an initial run), and 
you wish to run this Ansible playbook manually, you can add it to your host vars file.

`--ask-become-pass` is telling Ansible to prompt you for your sudo password, so it can run installs and other configuration tasks on your behalf.

`deploy_benchmarker.yml` is the name of our playbook that we want Ansible to run. 

## Using Tags to Control What is Deployed
Each part of the `deploy_benchmarker.yml` playbook has 
[tags](https://docs.ansible.com/ansible/latest/user_guide/playbooks_tags.html) associated with them. 
These tags allow us to tell Ansible which part(s) of the playbook we want to run. In other words, tags
allow us to tell Ansible which parts of the overall Logstash deployment pipeline we want to run.

They `deploy_benchmarker.yml` playbook (and a couple of roles) has the following tags in it:

* `init`
* `init_elk`
* `stop_elk`
* `prerequisites`
* `elk`
* `cdk`
* `run`
* `deploy`
* `destroy`
* `destroy_key_pair`
* `upload`
* `dynamodb`
* `dax`
* `crud`
* `read-only`

To view all these tags and their associated plays from the `ansible` CLI, run

```shell
ansible-playbook deploy_benchmarker.yml --list-tags
```

Using these tags, we can specify that we only want to run specific parts of the Benchmarking Deployment pipeline that's 
defined in the `deploy_benchmarker.yml` playbook.

For example: If we only wanted to start the ELK (Elasticsearch-Logstash-Kibana) stack, we would run this:

```shell
ansible-playbook -i inventories/local --tags elk deploy_benchmarker.yml
```

Likewise, if we wanted to stop the ELK stack, we'd run this:

```shell
ansible-playbook -i inventories/local --tags stop_elk deploy_benchmarker.yml
```

Note the `--tags` argument. This allows us to tell Ansible to only run tasks or roles that have the
`elk` or `stop_elk` tag on them.

We can also specify multiple arguments for `--tags` if we wish; for example, if we wanted to simply spin up the local
Elastic stack (synonymous with ELK stack), and deploy the CDK, we'd run the following:

```shell
ansible-playbook -i inventories/local -e vpc_id=vpc-1234567890 --tags 'elk,cdk' deploy_benchmarker.yml
```

## Plays
The following plays can be run from these playbooks using the tags with the following commands:

#### Initialize Your Local Environment and Elastic Stack
A sudo password is required to install applications, so we tell Ansible to prompt us for it at the start:

```shell
ansible-playbook -i inventories/local --tags init deploy_benchmarker.yml --ask-become-pass
```

#### Deploy CDK and Run the Benchmarkers on the Bastion Host
This assumes you already know the VPC ID to deploy into and have already created an SSH key pair and have the key pair 
locally in your `~/.ssh` directory with a `.pem` extension.

If you did not do this manually, it was done for you automatically and the created pair is under `~/.ssh/$USER-dax-pair.pem`.

You can either specify the `vpc_id` argument directly via `-e` in the command, or you can hard code
it in your [host_vars](./inventories/local/host_vars/localhost.yml). You must also already be logged into the AWS CLI for
your target environment, or specify a `profile_id` either in your `host_vars` or via `-e`, along with an `aws_region`. If you're not
already logged into AWS, your `profile_id` must be configured to be picked up automatically from your `~/.aws/config` or 
`~/.aws/credentials` files with no additional login steps in order to deploy to AWS.

```shell
ansible-playbook -i inventories/local -e vpc_id=vpc-1234567890 --tags deploy deploy_benchmarker.yml
```

#### Shut Down Your Local Elastic Stack
```shell
ansible-playbook -i inventories/local --tags stop_elk deploy_benchmarker.yml
```

#### Wipe Away everything
Once more, this assumes you either have the DAX
endpoint and the VPC ID hardcoded in your [host vars](./inventories/local/host_vars/localhost.yml), or you provide them via `-e`.

If you've already run a CDK deploy via Ansible, then you should not need to specify anything.

**Note:** For safety purposes, this will _not_ wipe away the `ssk_key_name` in your `~/.ssh` directory. If you specified
a pre-existing key to use for this deployment, it will not be touched. If you did not specify a key name, the automatically
generated key `$USER-dax-pair` will be left in your `~/.ssh` directory. If you wish to delete this pair from your local machine
and remove it from AWS, also specify the `destroy_key_pair` tag as well in the below command.

You can either specify the `vpc_id` argument directly via `-e` in the command, or you can hard code
it in your [host_vars](./inventories/local/host_vars/localhost.yml). You must also already be logged into the AWS CLI for
your target environment, or specify a `profile_id` either in your `host_vars` or via `-e`, along with an `aws_region`. If you're not
already logged into AWS, your `profile_id` must be configured to be picked up automatically from your `~/.aws/config` or
`~/.aws/credentials` files with no additional login steps in order to deploy to AWS.

**Destroy Everything, But Leave the ssh_key_name Key-Pair Alone:**
```shell
ansible-playbook -i inventories/local -e vpc_id=vpc-1234567890 --tags destroy deploy_benchmarker.yml
```

**Destroy Everything, Including the ssh_key_name Key-Pair**
```shell
ansible-playbook -i inventories/local -e vpc_id=vpc-1234567890 --tags 'destroy,destroy_key_pair' deploy_benchmarker.yml
```

### Additional Plays You Can Run

#### Only Install Prerequisites for Local Machine
A sudo password is required to install applications, so we tell Ansible to prompt us for it at the start:

```shell
ansible-playbook -i inventories/local --tags prerequisites deploy_benchmarker.yml --ask-become-pass
```

#### Start Your Local Elastic Stack
```shell
ansible-playbook -i inventories/local --tags elk deploy_benchmarker.yml
```

#### Just Deploy the CDK
This assumes you already know the VPC ID to deploy into and have already created an SSH key pair and have the key pair 
locally in your `~/.ssh` directory with a `.pem` extension. If you did not do this manually, it was done for you automatically
and the created pair is under `~/.ssh/$USER-dax-pair.pem`. You can either specify the `vpc_id` 
argument directly via `-e` in the command, or you can hard code it in your [host_vars](./inventories/local/host_vars/localhost.yml).

If you've already run a CDK deploy via Ansible, then you should not need to specify anything.

You must also already be logged into the AWS CLI for your target environment, or specify a `profile_id` either in your 
`host_vars` or via `-e`, along with an `aws_region`. If you're not already logged into AWS, your `profile_id` must be 
configured to be picked up automatically from your `~/.aws/config` or `~/.aws/credentials` files with no additional 
login steps in order to deploy to AWS.

```shell
ansible-playbook -i inventories/local --tags cdk deploy_benchmarker.yml
```

#### Only Upload the Benchmarkers to the Bastion Host
```shell
ansible-playbook -i inventories/local --tags upload deploy_benchmarker.yml
```

#### Run All Benchmarkers and Scenarios
This assumes the CDK is already deployed and an EC2 instance already exists. This also assumes you either have the DAX
endpoint and the VPC ID hardcoded in your [host vars](./inventories/local/host_vars/localhost.yml), or you provide them via `-e`. 
If you've already run a CDK deploy via Ansible, then you should not need to specify anything.

Additionally, You must also already be logged into the AWS CLI for
your target environment, or specify a `profile_id` either in your `host_vars` or via `-e`, along with an `aws_region`. If you're not
already logged into AWS, your `profile_id` must be configured to be picked up automatically from your `~/.aws/config` or
`~/.aws/credentials` files with no additional login steps in order to deploy to AWS:

```shell
ansible-playbook -i inventories/local --tags run run_benchmarkers.yml
```

#### Only Run the DynamoDB/DAX Benchmarker
This assumes the CDK is already deployed and an EC2 instance already exists. This also assumes you either have the DAX 
endpoint and the VPC ID hardcoded in your [host vars](./inventories/local/host_vars/localhost.yml), or you provide them via `-e`.  
If you've already run a CDK deploy via Ansible, then you should not need to specify anything.

Additionally, You must also already be logged into the AWS CLI for
your target environment, or specify a `profile_id` either in your `host_vars` or via `-e`, along with an `aws_region`. If you're not
already logged into AWS, your `profile_id` must be configured to be picked up automatically from your `~/.aws/config` or
`~/.aws/credentials` files with no additional login steps in order to deploy to AWS:

```shell
ansible-playbook -i inventories/local --tags dynamodb deploy_benchmarker.yml
```

or

```shell
ansible-playbook -i inventories/local --tags dax deploy_benchmarker.yml
```

Note the difference in tags: `dynamodb` and `dax`

#### Only Run the Benchmarkers in CRUD/READONLY mode
This assumes the CDK is already deployed and an EC2 instance already exists. This also assumes you either have the DAX
endpoint and the VPC ID hardcoded in your [host vars](./inventories/local/host_vars/localhost.yml), or you provide them via `-e`.  
If you've already run a CDK deploy via Ansible, then you should not need to specify anything.

Additionally, You must also already be logged into the AWS CLI for
your target environment, or specify a `profile_id` either in your `host_vars` or via `-e`, along with an `aws_region`. If you're not
already logged into AWS, your `profile_id` must be configured to be picked up automatically from your `~/.aws/config` or
`~/.aws/credentials` files with no additional login steps in order to deploy to AWS:

**CRUD:**
```shell
ansible-playbook -i inventories/local --tags crud deploy_benchmarker.yml
```

**read-only:**
```shell
ansible-playbook -i inventories/local --tags read-only deploy_benchmarker.yml
```

## Supported Variables
The following variables are supported to be specified via the `-e` argument when running the `deploy_benchmarker.yml` 
playbook:

| Variable Name     | Description                                                                                                                                   | Required? |
|-------------------|-----------------------------------------------------------------------------------------------------------------------------------------------|-----------|
| `profile_id`      | The name of the AWS CLI profile you wish to deploy with; <br>Defaults to using the `AWS_PROFILE` environment variable                         |           |
| `vpc_id`          | The ID of the VPC in the AWS account you're deploying to where you want the CDK components created <br>Only required on first run only        | *         |
| `local_ip`        | The public IP of your local machine; <br>Defaults to the response from `curl -s -L checkip.amazonaws.com`                                     |           |
| `ssh_key_name`    | The name of the SSH key-pair that will be used when creating the EC2 instance to allow you SSH access to it; <br>Defaults to `$USER-dax-pair` |           |
| `aws_account`     | The account ID of the AWS account you're deploying into; <br>Defaults to the result of `aws sts get-caller-identity \| jq -r .Account`        |           |
| `base_table_name` | The base name to use when creating the DynamoDB table; <br>Defaults to `high-velocity-table`                                                  |           |
| `cdk_action`      | The action to perform when deploying the CDK; <br>Defaults to `deploy`                                                                        |           |
| `duration`        | How long to run each simulation for; <br>Defaults to 1800 seconds                                                                             |           |
| `benchmarker`     | Which benchmarker to run (i.e. `dynamodb` or `dax`)                                                                                           |           |
| `dax_endpoint`    | The DAX URI to use to hit the DAX cluster; <br>Only required when running the benchmarkers and without an initial CDK deploy)                 | *         |

## Run Order
When first running from scratch, you'll want to run with the `init` tags first to initialize the Elastic Stack and install the prerequisites, then run again without any tags to actually
deploy everything and run the benchmarkers. If you only want to run the benchmarkers, run the `run_benchmarkers.yml` playbook, or specify the `run` tag.

## Troubleshooting
You can generally get more information about your problem by adding `-vvv` to the end of your
`ansible-playbook` command. The more `v`'s you add, the more verbose the output and the more information
you will get. For example:

```shell
ansible-playbook -i inventories/local -e cdk_action=destroy --tags 'elk,cdk' deploy_benchmarker.yml -vvv
```