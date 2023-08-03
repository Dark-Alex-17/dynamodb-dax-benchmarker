#!/bin/bash
export PATH="$HOME"/.local/bin:$PATH

source scripts/logger.sh /tmp/benchmarker-tui.log
source scripts/ui_utils.sh

ANSIBLE_LOG_FILE=/tmp/ansible-playbook-output.log
rm "$ANSIBLE_LOG_FILE" > /dev/null 2>&1 &

verify-prerequisites() {
	log-info "Verifying prerequisites"
	declare prerequisites=(whiptail jq dialog)

	if ! (aws sts get-caller-identity > /dev/null 2>&1); then
    log-error "Must be logged into AWS CLI to use this script. Log into the target AWS account and run this script again" true
    exit 1
  fi

	for application in "${prerequisites[@]}"; do
		if ! (command -v "$application" > /dev/null 2>&1); then
			log-warn "$application is required to run this script. Installing $application..."
			sudo apt install "$application"
		fi
	done

	if ! (command -v ansible > /dev/null 2>&1); then
  	log-warn "Ansible is required to run this script. Installing Ansible..." true
  	sudo apt install python3-pip
  	pip3 install --user ansible jmespath
  fi

  if ! (ansible-galaxy collection list | grep -i "community.general\|amazon.aws" > /dev/null 2>&1); then
  	log-warn "Installing Ansible galaxy requirements..." true
  	cd ansible
  	ansible-galaxy install -r requirements.yml
  	cd -
  fi
}

initialize-environment() {
	check-sudo-pass "Installing dependencies requires sudo permissions."
	if [[ "$?" == 0 ]]; then
		declare title="Initialize Local Environment"

		if (prompt-yes-no "$title"); then
			cd ansible

			ansible-playbook -i inventories/local -e "ansible_become_password=$PASSWORD" --tags init deploy_benchmarker.yml > "$ANSIBLE_LOG_FILE" 2>&1 &
			pid=$!
			log-info "Running ansible-playbook 'deploy_benchmarker.yml' with the 'init' tag and logging output to file [$ANSIBLE_LOG_FILE]"

			show-tail-box "$title" $pid "$ANSIBLE_LOG_FILE"

			msg-box "Successfully initialized the local environment!"
			log-info "Successfully initialized the local environment"

			cd -
		fi
	fi

	main-menu
}

prompt-for-vpc-id() {
	readarray -t vpc_arr < <(aws ec2 describe-vpcs | jq -r '.Vpcs[] | "\(.VpcId) \((.Tags[]? | select(.Key | contains("Name")) | .Value) // "")"' | awk '{print($1, $2 == "" ? "-" : $2);}')
	declare prompt=""
	for item in "${vpc_arr[@]}"; do
		prompt+="$item OFF "
	done

	VPC_ID=$(whiptail --fb --title "Select VPC" --radiolist "Select which VPC to use to deploy resources into" "$BOX_HEIGHT" "$BOX_WIDTH" "${#vpc_arr[@]}" $prompt 3>&2 2>&1 1>&3)
}

deploy-and-run-benchmarkers() {
	declare title="Deploy and Run Benchmarkers"
	
	if (prompt-yes-no "$title"); then
		if [[ -z $VPC_ID ]]; then
			prompt-for-vpc-id
		fi

		cd ansible

		ansible-playbook -i inventories/local -e vpc_id="$VPC_ID" --tags deploy deploy_benchmarker.yml > "$ANSIBLE_LOG_FILE" 2>&1 &
		pid=$!
		log-info "Running ansible-playbook 'deploy_benchmarker.yml' with tags [deploy] and logging output to file [$ANSIBLE_LOG_FILE]"

		show-tail-box "$title" $pid "$ANSIBLE_LOG_FILE"

		msg-box "Successfully deployed and ran benchmarkers!"
		log-info "Successfully deployed and ran benchmarkers"

		cd -
	fi

	main-menu
}

destroy-all() {
	declare title="Destroy Everything (Clean Slate)"
	
	if (prompt-yes-no "$title"); then
		cd ansible

		ansible-playbook -i inventories/local --tags 'destroy,destroy_key_pair' deploy_benchmarker.yml > "$ANSIBLE_LOG_FILE" 2>&1 &
		pid=$!
		log-info "Running ansible-playbook 'deploy_benchmarker.yml' with [destroy,destroy_key_pair] tags and logging output to file [$ANSIBLE_LOG_FILE]"

		show-tail-box "$title" $pid "$ANSIBLE_LOG_FILE"

		msg-box "Successfully destroyed everything!"
		log-info "Successfully destroyed everything"

		cd -
	fi

	main-menu
}

randomly-populate-dynamodb() {
	declare title="Populate DynamoDB with Random Data"
	declare log_file=/tmp/dynamodb-data-population.log
	
	if (prompt-yes-no "$title"); then
		./scripts/randomly-generate-high-velocity-data.sh -i 5000 > "$log_file" 2>&1 &
		pid=$!
		log-info "Running randomly-generate-high-velocity-data script and logging to [$log_file]"

		show-tail-box "$title" $pid "$log_file"

		msg-box "Successfully populated DynamoDB with random data!"
		log-info "Successfully populated DynamoDB with random data"
	fi

	main-menu
}

custom-selections() {
	declare title="Customize What to Run (Advanced Mode)"
	declare choices
	declare prompted_for_sudo_pass=false
	declare prompted_for_vpc_id=false
	declare requires_vpc_id=false
	declare tags=""

	choices=$(whiptail --separate-output --checklist --fb "$title" "$CHECKBOX_HEIGHT" "$CHECKBOX_WIDTH" 13 \
		"PREREQUISITES" "Install Prerequisites for Local Machine" OFF \
		"INITIALIZE_ELK" "Initialize Local Elastic Stack" OFF \
		"START_ELK" "Start Local Elastic Stack" OFF \
		"DEPLOY_CDK" "Deploy CDK" OFF \
		"UPLOAD_BIN" "Upload Benchmarker binaries" OFF \
		"RUN_BENCHMARKERS" "Run Benchmarkers" OFF \
		"STOP_ELK" "Stop Local Elastic Stack" OFF \
		"DESTROY" "Destroy Everything except the SSH key" OFF \
		"DESTROY_KEY" "Destroy the SSK Key" OFF \
		"RUN_DYNAMODB" "Run the DynamoDB Benchmarkers" OFF \
		"RUN_DAX" "Run the DAX Benchmarkers" OFF \
		"RUN_CRUD" "Run the CRUD benchmarks for both the DynamoDB and DAX benchmarkers" OFF \
		"RUN_READ_ONLY" "Run the READ-ONLY benchmarks for both the DynamoDB and DAX benchmarkers" OFF 3>&2 2>&1 1>&3)

	if [[ -n $choices ]]; then
		for choice in $choices; do
			case "$choice" in
				"PREREQUISITES")
					tags+="prerequisites,"
					check-sudo-pass "Installing dependencies requires sudo permissions."
					prompted_for_sudo_pass=true
					;;
				"INITIALIZE_ELK")
					tags+="init_elk,"
					;;
				"START_ELK")
					tags+="elk,"
					;;
				"DEPLOY_CDK")
					tags+="cdk,"
					requires_vpc_id=true
					if [[ -z $VPC_ID ]]; then
						prompt-for-vpc-id
						prompted_for_vpc_id=true
					fi
					;;
				"UPLOAD_BIN")
					tags+="upload,"
					;;
				"RUN_BENCHMARKERS")
					tags+="run,"
					requires_vpc_id=true
					if [[ -z $VPC_ID ]]; then
						prompt-for-vpc-id
						prompted_for_vpc_id=true
					fi
					;;
				"STOP_ELK")
					tags+="stop_elk,"
					;;
				"DESTROY")
					tags+="destroy,"
					;;
				"DESTROY_KEY")
					tags+="destroy_key_pair,"
					;;
				"RUN_DYNAMODB")
					tags+="dynamodb,"
					;;
				"RUN_DAX")
					tags+="dax,"
					;;
				"RUN_CRUD")
					tags+="crud,"
					;;
				"RUN_READ_ONLY")
					tags+="read-only,"
					;;
			esac
		done

		tags=$(echo "$tags" | sed 's/\(.*\),/\1/')


		if (prompt-yes-no "Advanced Mode: Deploy tasks with tags: [$tags]"); then
			cd ansible
			args=""

			if [[ $prompted_for_sudo_pass == true ]]; then
				args+=" -e ansible_become_password='$PASSWORD'"
			fi

			if [[ $requires_vpc_id == true && $prompted_for_vpc_id == true ]]; then
				args+=" -e vpc_id=$VPC_ID"
			fi

			ansible-playbook -i inventories/local $args --tags "$tags" deploy_benchmarker.yml > "$ANSIBLE_LOG_FILE" 2>&1 &
			pid=$!
			log-info "Running ansible-playbook 'deploy_benchmarker.yml' with [$tags] tags and logging output to file [$ANSIBLE_LOG_FILE]"

			show-tail-box "$title" $pid "$ANSIBLE_LOG_FILE"

			msg-box "Successfully ran custom tasks!"
			log-info "Successfully ran custom tasks"

			cd -
		fi
	fi

	main-menu
}

main-menu() {
	declare choice
	choice=$(whiptail --fb --title "DynamoDB + DAX Benchmarker" --menu "Select an action" "$BOX_HEIGHT" "$BOX_WIDTH" 7 \
		"I" "(I)nitialize local environment" \
		"B" "Open Dash(b)oards" \
		"D" "(D)eploy and Run benchmarkers" \
		"W" "(W)ipe away everything (Clean Slate)" \
		"R" "(R)andomly populate DynamoDB" \
		"C" "(C)ustom (Advanced)" \
		"X" "E(x)it" 3>&2 2>&1 1>&3)

	case $choice in
		"I")
			initialize-environment
			;;
		"B")
			msg-box "$(cat <<-EOF
				This will open two new tabs in Firefox to display the DynamoDB and DAX dashboards.

				The default credentials are:
				Username: elastic
				Password: changeme
			EOF
			)"
			firefox --new-tab --url 'http://localhost:5601/app/dashboards#/view/51721040-24be-11ee-ac2e-ff8a2f0e28da?_g=(filters:!(),refreshInterval:(pause:!t,value:60000),time:(from:now-15m,to:now))' > /dev/null 2>&1 &
			firefox --new-tab --url 'http://localhost:5601/app/dashboards#/view/0fe18820-2736-11ee-a70f-4976799912d8?_g=(filters:!(),refreshInterval:(pause:!t,value:60000),time:(from:now-15m,to:now))' > /dev/null 2>&1 &
			;;
		"D")
			deploy-and-run-benchmarkers
			;;
		"W")
			destroy-all
			;;
		"R")
			randomly-populate-dynamodb
			;;
		"C")
			msg-box "This is for advanced users only! Be sure you know what you're doing, as running some things at the same time can cause problems (like destroy and deploy)!"
			custom-selections
			;;
		"X")
			clear
			exit 0
			;;
	esac
}

verify-prerequisites

while :; do
	main-menu
done