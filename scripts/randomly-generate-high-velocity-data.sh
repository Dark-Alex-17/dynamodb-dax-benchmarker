#!/bin/bash
if [[ $(basename "$(pwd)") == scripts ]]; then
  source logger.sh
else
  source scripts/logger.sh
fi

trap 'echo Stopping...; exit' SIGINT

usage() {
  cat << EOF
${blue}${bold}randomly-generate-high-velocity-data${default}: A script to randomly generate high-velocity data for some DynamoDB table with random attributes and values for benchmarking purposes.

${gold}${bold}USAGE:${default}
  randomly-generate-high-velocity-data [OPTIONS] [ARGS]...

  ${green}-h, --help${default}                           Show this usage screen

${gold}${bold}ARGS:${default}
  ${green}-a, --attributes ${magenta}<ATTRIBUTES>${default}        The number of attributes to populate each item in the table with
                                       This defaults to 5

  ${green}-i, --items      ${magenta}<ITEMS>${default}             The number of items to populate the table with
                                       ${bold}Note:${default} Items are populated 25 at a time, so whatever number you provide will be rounded to the nearest multiple of 25

  ${green}-t, --table      ${magenta}<TABLE_NAME>${default}        The name of the DynamoDB table to populate
                                       This defaults to $USER-high-velocity-table
EOF
}

ensure-required-variables-are-set() {
	declare required_variables=(AWS_PROFILE AWS_REGION ITEMS)

	for variable in "${required_variables[@]}"; do
		if [[ -z "${!variable}" ]]; then
			log-error "A required variable environment is not initialized: $variable"
			exit 1
		fi
	done
}

parse-arguments() {
  declare parsed_args
  parsed_args=$(getopt -a -n randomly-generate-high-velocity-data -o :a:hi:t: --long attributes:,help,items:,table: -- "$@")
  declare valid_arguments=$?

  if [[ $valid_arguments != 0 ]]; then
    log-error "Invalid arguments passed. See usage below."
    usage
    exit 1
  fi

  eval set -- "$parsed_args"
  while :; do
    case "$1" in
      "-a" | "--attributes")
        ATTRIBUTES="$2"
        shift 2
        ;;
      "-h" | "--help")
        usage
        exit
        ;;
			"-i" | "--items")
				ITEMS="$2"
				shift 2
				;;
      "-t" | "--table")
        TABLE_NAME="$2"
        shift 2
        ;;
      --)
        shift
        break
        ;;
      *)
        log-error "An invalid option was passed, but somehow getopt didn't catch it: $1. Displaying usage and exiting..."
        usage
        exit 1
        ;;
    esac
  done

	if [[ -z $TABLE_NAME ]]; then
		TABLE_NAME="$USER-high-velocity-table"
	fi

	if [[ -z $ATTRIBUTES ]]; then
		ATTRIBUTES=5
	fi

  ensure-required-variables-are-set

	if [[ $ATTRIBUTES -lt 1 ]]; then
		log-error "ATTRIBUTES must be a value of at least 1 so that attributes can be added to the table."
		exit 1
	fi

	if ! (aws sts get-caller-identity > /dev/null 2>&1); then
		log-error "You must be logged into the AWS CLI in order to use this script. Please log into the AWS CLI first and then try again."
		exit 1
	fi
}

show-properties() {
  log-info "Using the following settings to randomly populate the DynamoDB benchmarking table:"
  cat <<-EOF
	${cyan}
	ATTRIBUTES=$ATTRIBUTES
	TABLE_NAME=$TABLE_NAME
	${default}
	EOF
}

generate-attribute-value() {
	declare current_val=$1
	case "$((current_val % 2))" in
		"1")
			echo '"'"$current_val"'": {"N": "'"$(seq 0 .01 32 | shuf | head -1)"'"}'
			;;
		*)
			echo '"'"$current_val"'": {"S": "'"$(base64 /dev/urandom | awk '{print(0==NR%100)?"":$1}' | sed 's/[^[:alpha:]]/ /g' | head -1)"'"}'
			;;
	esac
}

generate-put-request() {
	declare attribute_values
	attribute_values=$(generate-attribute-value 0)

	for j in $(seq 1 $((ATTRIBUTES-1))); do
		attribute_values="$attribute_values, $(generate-attribute-value "$j")"
	done

	
	cat <<-EOF
    {
      "PutRequest": {
        "Item": {
          "id": {"S": "$(cat /proc/sys/kernel/random/uuid)"},
					$attribute_values
				}
			}
		}
	EOF
}

generate-batch-json() {
  declare batch_request='{ "'"$TABLE_NAME"'": ['
	batch_request="$batch_request $(generate-put-request)"

  for i in $(seq 0 23); do
    batch_request="$batch_request, $(generate-put-request)"
  done

  batch_request="$batch_request ]}"

  echo "$batch_request"
}

if ! (command -v aws > /dev/null 2>&1); then
	log-error "The AWS CLI must be installed first. Install the CLI first and then try again."
	exit 1
fi

parse-arguments "$@"
show-properties

declare -i i=0
declare -i items_written=0
while [[ $items_written -lt $ITEMS ]]; do
  log-info "Writing 25 entries to DynamoDB..."
  aws dynamodb batch-write-item --request-items "$(generate-batch-json)"
  log-info 'Entries Written!'
  ((i++))
	((items_written+=25))
  log-info "Total entries written: $items_written"
  log-info "Sleeping for 2 seconds to avoid the partition throughput limits..."
  sleep 2
done
