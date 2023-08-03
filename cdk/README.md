# DynamoDB + DAX Benchmarker CDK

This CDK project deploys a DynamoDB table with a DAX cluster on top of it, and an EC2 instance to act as a bastion host for running benchmarking tests agasint DAX.

By default, the name of the DynamoDB table that is created is `$USER-high-velocity-table`.
By default, the name of the SSH key that is created for you is `$USER-dax-pair`

It should be noted that due to a bug in CDK, if you destroy the stack, you'll have to manually delete the SubnetGroup in DAX once everything else is deleted.

## Prerequisites
You must be logged into the AWS CLI prior to running the CDK. Ensure you're logged into your target AWS account by running 
`aws sts get-caller-identity`.

## Getting started
[NodeJS](https://nodejs.org/en) is required for development. Install NodeJS using the following commands, if it is 
not already installed:

### Installing NodeJS

#### Windows
NodeJS can be installed on Windows using the [Chocolatey](https://chocolatey.org) package manager. If Chocolatey is not yet
installed on your system, first install it in a privileged PowerShell:
```powershell
Set-ExecutionPolicy Bypass -Scope Process -Force;
    [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; 
    iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))
```

Then, in a _non-privileged_ PowerShell session, install node:
```powershell
choco install nodejs
```

#### Linux
NodeJS can be installed on Linux using [NVM](https://github.com/nvm-sh/nvm). First, install NVM:
```shell
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.3/install.sh | bash
```

**Note:** The installation command was _not_ run with `sudo`. This is intentional, because if you install with `sudo`, then
`sudo` permissions will be required to install any and all new dependencies! You should avoid installing Node for the root
user!


Then, in order to use NVM to install NodeJS, you need to either restart your current shell session, or run the following:
```shell
export NVM_DIR="$([ -z "${XDG_CONFIG_HOME-}" ] && printf %s "${HOME}/.nvm" || printf %s "${XDG_CONFIG_HOME}/nvm")"
[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"
```

Now, install NodeJS:
```shell
nvm install node 
```

### Installing dependent libraries

Once node is installed, run the following commands to install the NPM libraries:

```shell
cd cdk
npm install -g aws-cdk
npm install -g typescript --save-dev
npm install
```

## CDK Arguments
This application depends on a few additional parameters in order to run. They can be specified in one of two ways: environment variables, or via the `-c` argument of the `cdk` command.

**Important:** Only one environment variable is required by the application, regardless of which parameter specification method you choose: `AWS_REGION`.

The following is a table of the **required** parameters for running the CDK

| Parameter Name | Environment Variable Name | Description                                                                                                                                                          |
|----------------|---------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `vpcId`        | `VPC_ID`                  | The VPC ID you wish to deploy all of the stack's components into                                                                                                     |
| `localIp`      | `LOCAL_IP`                | Your local IP; Used to allow SSH and Elasticsearch access in the EC2 security group                                                                                  |
| `sshKeyName`   | `SSH_KEY_NAME`            | The key name of your ssh key to allow you access to your EC2 instance. This should only be the name of the `.pem` file, and should not include the `.pem` extension. |
| `awsAccount`   | `AWS_ACCOUNT`             | The account ID of your AWS account.                                                                                                                                  |
| `awsRegion`    | `AWS_REGION`              | The AWS region to deploy this stack and its components into                                                                                                          |

### Optional Parameters
It is sometimes necessary to tweak the deployment a bit for different use cases. The CDK can be tweaked with the following parameters:

| Parameter Name  | Default Value         | Description                                                                                                             |
|-----------------|-----------------------|-------------------------------------------------------------------------------------------------------------------------|
| `baseTableName` | `high-velocity-table` | This is the base name for the table. All tables created by the stack will be prefixed with `$USER` to prevent conflicts |

