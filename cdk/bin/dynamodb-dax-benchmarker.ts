#!/usr/bin/env node
import 'source-map-support/register';
import * as cdk from 'aws-cdk-lib';
import {EnvironmentProps} from '../lib/types';
import { DaxBenchmarkingStack } from '../lib/dax-benchmarking-stack';

const app = new cdk.App();
const user = process.env.USER || '';
let vpcId = app.node.tryGetContext('vpcId');
if (!vpcId) {
	if (!process.env.VPC_ID) {
			throw new Error('vpcId is a required parameter. Specify it with `-c vpcId=someId`, or by setting the VPC_ID environment variable');
	} else {
		vpcId = process.env.VPC_ID
	}
}

let localIp = app.node.tryGetContext('localIp');
if (!localIp) {
	if (!process.env.LOCAL_IP) {
			throw new Error('Local IP is a required parameter. Specify it with `-c localIp=XXX.XXX.XXX.XXX`, or by setting the LOCAL_IP environment variable');
	} else {
		localIp = process.env.LOCAL_IP
	}
}

let sshKeyName = app.node.tryGetContext('sshKeyName');
if (!sshKeyName) {
	if (!process.env.SSH_KEY_NAME) {
		sshKeyName = `${user}-dax-pair`;
	} else {
		sshKeyName = process.env.SSH_KEY_NAME;
	}
}

let awsAccount = app.node.tryGetContext('awsAccount');
if (!awsAccount) {
	if (!process.env.AWS_ACCOUNT) {
		throw new Error('awsAccount is a required parameter. Specify it with `-c awsAccount=1234567890`, or by setting the AWS_ACCOUNT environment variable.');
	} else {
		awsAccount = process.env.AWS_ACCOUNT;
	}
}

let awsRegion = app.node.tryGetContext('awsRegion');
if (!awsRegion) {
	if (!process.env.AWS_REGION) {
		throw new Error('The `AWS_REGION` environment variable was not set. It must be set in order to use this application.');
	} else {
		awsRegion = process.env.AWS_REGION
	}
}

let baseTableName = app.node.tryGetContext('baseTableName');
if (!baseTableName) {
	baseTableName = 'high-velocity-table'
}

const environmentProps: EnvironmentProps = {
	env: { account: awsAccount, region: awsRegion },
	baseTableName,
	removalPolicy: cdk.RemovalPolicy.DESTROY,
	user,
	vpcId,
	localIp,
	sshKeyName
};

new DaxBenchmarkingStack(app, `${user}-dax-benchmark-stack`, environmentProps);
