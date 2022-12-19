#!/bin/bash

#Help
if [[ "$1" == "-h" ]] 
then
  printf "Description: Get the serial id from stm32wb55rg. The output will be in the result file\n"
  exit 0
fi

#Set variables
sudo rm result
sudo rm debug
. ./wb55_helper_functions.sh --source-only

#main function
sudo openocd & (sleep 1 &&
(
    sleep 1
    execute_command "reset halt" "xpsr" "not" "not" "Unable to halt"
    echo "init"
    sleep 0.1

    #Get UUID
    echo "mdb 0x1fff7590 12"
    sleep 0.1
    output=$(cat debug | tr -d '\0' | grep -a '0x1fff7590' | tail -1)
    value=$(cut -d ":" -f2 <<< $output)
    value=$(echo $value | sed -e 's/ //g')
    
    echo "$value" > result
    echo "Success" >> result
    #Reset and exit
    echo "reset"
    sleep 1
    echo "exit"
) |  telnet localhost 4444  > debug)
sudo pkill openocd



