import { Construct } from "constructs";
import { EnvironmentProps } from "./types";
import { CfnOutput, Stack, Tags } from "aws-cdk-lib";
import { CfnCluster, CfnSubnetGroup } from "aws-cdk-lib/aws-dax";
import { Effect, PolicyDocument, PolicyStatement, Role, ServicePrincipal } from "aws-cdk-lib/aws-iam";
import { SecurityGroup, SubnetType, Vpc } from "aws-cdk-lib/aws-ec2";
import { DynamoDbBenchmarkTable } from "./dynamodb";
import { DaxBastionHost } from "./bastion-host";

export class DaxBenchmarkingStack extends Stack {
  constructor(scope: Construct, id: string, environmentProps: EnvironmentProps) {
    super(scope, id, environmentProps);

    Tags.of(this).add('Application', 'dynamodb-dax-benchmarker');

    const { user, removalPolicy, vpcId } = environmentProps;
    const { table } = new DynamoDbBenchmarkTable(this, `${user}-dynamodb-benchmark-table`, environmentProps);

    const vpc = Vpc.fromLookup(this, 'Vpc', { vpcId });

    const daxSecurityGroup = new SecurityGroup(this, `${user}-dax-sg`, {
      vpc,
      securityGroupName: `${user}-dax-sg`
    });
    daxSecurityGroup.applyRemovalPolicy(removalPolicy);

    const { instanceRole, instance } = new DaxBastionHost(this, `${user}-dax-bastion-host`, environmentProps, daxSecurityGroup);

    const daxClusterName = `${user}-high-velocity`;
    const daxFullAccessPolicy = new PolicyStatement({
        effect: Effect.ALLOW,
        actions: [
            "dynamodb:BatchGetItem",
            "dynamodb:GetItem",
            "dynamodb:Query",
            "dynamodb:Scan",
            "dynamodb:BatchWriteItem",
            "dynamodb:DeleteItem",
            "dynamodb:PutItem",
            "dynamodb:UpdateItem",
            "dynamodb:DescribeLimits",
            "dynamodb:DescribeTimeToLive",
            "dynamodb:DescribeTable",
            "dynamodb:ListTables"
        ],
        resources: [table.tableArn]
    });

    const daxServiceRole = new Role(this, `${daxClusterName}-role`, {
      assumedBy: new ServicePrincipal("dax.amazonaws.com"),
      inlinePolicies: {
        DAXFullAccess: new PolicyDocument({
          statements: [daxFullAccessPolicy]
        })
      } 
    });
    daxServiceRole.applyRemovalPolicy(removalPolicy);

    instanceRole.addToPrincipalPolicy(daxFullAccessPolicy);

    const subnetGroup = new CfnSubnetGroup(this, `${user}-dax-subnet-group`, {
      subnetIds: vpc.selectSubnets({
        subnetType: SubnetType.PRIVATE_ISOLATED
      }).subnetIds,
      subnetGroupName: `${user}-dax-subnet-group`,
    });
    subnetGroup.applyRemovalPolicy(removalPolicy);
    
   const daxCluster = new CfnCluster(this, daxClusterName, {
      iamRoleArn: daxServiceRole.roleArn,
      nodeType: 'dax.r5.large',
      replicationFactor: 3,
      securityGroupIds: [daxSecurityGroup.securityGroupId],
      subnetGroupName: subnetGroup.subnetGroupName,
      availabilityZones: vpc.availabilityZones,
      clusterEndpointEncryptionType: 'TLS',
      clusterName: daxClusterName,
      sseSpecification: {
        sseEnabled: true,
      }
    });
    daxCluster.applyRemovalPolicy(removalPolicy);
    daxCluster.addDependency(subnetGroup);

    new CfnOutput(this, 'DaxEndpoint', { value: daxCluster.attrClusterDiscoveryEndpointUrl });
    new CfnOutput(this, 'InstanceId', { value: instance.instanceId });
    new CfnOutput(this, 'InstancePublicIp', { value: instance.instancePublicIp });
  }
}
