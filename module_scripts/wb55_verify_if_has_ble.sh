#!/bin/bash

#Help
if [[ "$1" == "-h" ]] 
then
  printf "Description: Verify if stm32wb55rg already has ble stack installed by verifying the SFSA. The output will be in the result file\n"
  exit 0
fi

#Set variables
sudo rm result
sudo rm debug
. ./wb55_helper_functions.sh --source-only

#main function
sudo openocd & (sleep 1 &&
(
    #sleep 1
    #echo "reset_config connect_deassert_srst"
    sleep 1
    execute_command "reset halt" "xpsr" "not" "not" "Unable to halt"
    echo "init"
    sleep 0.1

    #Verify option bytes
    echo "stm32wbx option_read 0 0x80"
    sleep 0.1
    output=$(cat debug | tr -d '\0' | grep -a '0x58004080' | tail -1)
    if [[ $output != *"e1"* ]]
    then
        echo "BLE stack not found" >> result
    else
        echo "BLE stack probably installed" >> result
    fi

    #Reset and exit
    echo "reset"
    sleep 1
    echo "exit"
) |  telnet localhost 4444  > debug) 
sudo pkill openocd



