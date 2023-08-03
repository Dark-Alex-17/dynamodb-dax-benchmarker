#!/bin/bash
TERMINAL_HEIGHT=$(tput lines)
BOX_HEIGHT=$(printf "%.0f" "$(echo "scale=2; $TERMINAL_HEIGHT * .5" | bc)")

TERMINAL_WIDTH=$(tput cols)
BOX_WIDTH=$(printf "%.0f" "$(echo "scale=2; $TERMINAL_WIDTH * .75" | bc)")

msg-box() {
	whiptail --fb --msgbox "$1" "$BOX_HEIGHT" "$BOX_WIDTH"
}

check-sudo-pass() {
	log-info "Prompting user for sudo password with message: $1"
	if [[ ! "$PASSWORD" ]]; then
		PASSWORD=$(whiptail --fb --passwordbox "$1 Enter your sudo password" "$BOX_HEIGHT" "$BOX_WIDTH" 3>&2 2>&1 1>&3)
	fi
}

show-tail-box() {
	trap "kill $2 2> /dev/null" EXIT

	while kill -0 "$2" 2> /dev/null; do
		dialog --title "$1" --exit-label "Finished" --tailbox "$3" "$BOX_HEIGHT" "$BOX_WIDTH"
	done

	clear

	trap - EXIT
}

prompt-yes-no() {
  declare action="$1"
  log-info "Prompting user if they wish to proceed with $action"

  whiptail --fb --title "$action?" --yesno "Are you sure you wish to proceed with the specified action: $action?" --defaultno "$BOX_HEIGHT" "$BOX_WIDTH"
}