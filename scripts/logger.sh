#!/bin/bash
BENCHMARK_LOG_FILE="$1"
if [[ -z $BENCHMARK_LOG_FILE ]]; then
	BENCHMARK_LOG_FILE=/tmp/benchmarker.log
fi

red=$(tput setaf 1)
green=$(tput setaf 2)
gold=$(tput setaf 3)
blue=$(tput setaf 4)
magenta=$(tput setaf 5)
cyan=$(tput setaf 6)
default=$(tput sgr0)
bold=$(tput bold)

log-error() {
  if [[ -z $2 ]]; then
    echo -e "${red}${bold}ERROR:${default}${red} $1${default}"
  else
    echo -e "${red}${bold}ERROR:${default}${red} $1${default}"
    echo -e "${red}${bold}ERROR:${default}${red} $1${default}" >> "$BENCHMARK_LOG_FILE"
  fi
}

log-warn() {
  if [[ -z $2 ]]; then
    echo -e "${gold}${bold}WARN:${default}${gold} $1${default}"
  else
    echo -e "${gold}${bold}WARN:${default}${gold} $1${default}"
    echo -e "${gold}${bold}WARN:${default}${gold} $1${default}" >> "$BENCHMARK_LOG_FILE"
  fi
}

log-info() {
  if [[ -z $2 ]]; then
    echo -e "${cyan}${bold}INFO:${default}${cyan} $1${default}"
  else
    echo -e "${cyan}${bold}INFO:${default}${cyan} $1${default}"
    echo -e "${cyan}${bold}INFO:${default}${cyan} $1${default}" >> "$BENCHMARK_LOG_FILE"
  fi
}
