import {Tags} from "aws-cdk-lib";
import {Construct} from "constructs";
import {EnvironmentProps} from "./types";
import {AttributeType, BillingMode, Table} from "aws-cdk-lib/aws-dynamodb";

export class DynamoDbBenchmarkTable extends Construct {
	public readonly table: Table;

	constructor(scope: Construct, id: string, environmentProps: EnvironmentProps) {
		super(scope, id);

		Tags.of(this).add('Application', 'dynamodb-dax-benchmarker');

		const { baseTableName, removalPolicy, user } = environmentProps;
		const tableName = `${user}-${baseTableName}`;

		this.table =  new Table(this, tableName, {
			partitionKey: {
				name: 'id',
				type: AttributeType.STRING
			},
			tableName,
			removalPolicy,
			billingMode: BillingMode.PAY_PER_REQUEST
		});
	}
}