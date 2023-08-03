import { Tags } from "aws-cdk-lib";
import { Construct } from "constructs";
import { EnvironmentProps } from "./types";
import { Instance, InstanceClass, InstanceSize, InstanceType, MachineImage, Peer, Port, SecurityGroup, SubnetType, Vpc } from "aws-cdk-lib/aws-ec2";
import { IRole, Role, ServicePrincipal } from "aws-cdk-lib/aws-iam";

export class DaxBastionHost extends Construct {
  public readonly instanceRole: IRole;
  public readonly instance: Instance;

  constructor(scope: Construct, id: string, environmentProps: EnvironmentProps, daxSecurityGroup: SecurityGroup) {
    super(scope, id);

    Tags.of(this).add('Application', 'dynamodb-dax-benchmarker');

    const { removalPolicy, user, vpcId, localIp, sshKeyName } = environmentProps;
    const localIpCidr = `${localIp}/32`;

    const vpc = Vpc.fromLookup(this, 'Vpc', { vpcId });

    const bastionHostSecurityGroup = new SecurityGroup(this, `${user}-dax-sg`, {
      vpc,
      description: `Allow SSH, Elasticsearch, and DAX access for ${user}`,
      securityGroupName: `${user}-dax-bastion-host-sg`
    });
    bastionHostSecurityGroup.applyRemovalPolicy(removalPolicy);
    bastionHostSecurityGroup.addIngressRule(Peer.ipv4(localIpCidr), Port.tcp(22), "Allow SSH access to this instance from the users public IP");
    bastionHostSecurityGroup.addIngressRule(Peer.ipv4(localIpCidr), Port.tcp(9200), "Allow the host to communicate with the users locally running Elasticsearch cluster");
    bastionHostSecurityGroup.addIngressRule(daxSecurityGroup, Port.allTraffic());
    daxSecurityGroup.addIngressRule(bastionHostSecurityGroup, Port.allTraffic());

    this.instanceRole = new Role(this, `${user}-bastion-role`, {
      roleName: `${user}-bastion-role`,
      assumedBy: new ServicePrincipal('ec2.amazonaws.com'),
    });
    this.instanceRole.applyRemovalPolicy(removalPolicy);

    this.instance = new Instance(this, `${user}-dax-bastion-host`, {
      vpc,
      instanceType: InstanceType.of(InstanceClass.T2, InstanceSize.SMALL),
      machineImage: MachineImage.latestAmazonLinux2023(),
      instanceName: `${user}-dax-bastion-host`,
      keyName: sshKeyName,
      vpcSubnets: vpc.selectSubnets({ subnetType: SubnetType.PUBLIC }),
      securityGroup: bastionHostSecurityGroup,
      role: this.instanceRole
    });
    this.instance.applyRemovalPolicy(removalPolicy);
  }
}
