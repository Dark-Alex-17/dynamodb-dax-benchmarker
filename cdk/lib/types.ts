import {RemovalPolicy, StackProps} from "aws-cdk-lib";

export interface EnvironmentProps extends StackProps {
	readonly baseTableName: string
	readonly removalPolicy: RemovalPolicy
	readonly user: string
	readonly vpcId: string
	readonly localIp: string
	readonly sshKeyName: string
}
