#!/bin/bash

#Help
if [[ "$1" == "-h" ]]
then
  printf "Description: This script can install the required program on the stm32wb55rg, read the memory or reset the device. The output will be in the result file
  Usage: This script expect 2 arguments: command, [input_file], where:
   \t command = program to program a binary to the memory, compare to read a binary from memory and compare with the file passed, reset to reset the device
   \t file = Input file if program (necessary be a binary (.bin)), output file if read\n"
  exit 0
fi

#Warning about arguments
if [[ $# -eq 0 ]]
then
    echo "command expected"
    exit 0
fi

if [[ $1 != *"reset"* ]] && [[ $# -ne 2 ]]
then
    echo "binary file expected"
    exit 0
fi

#Set variables
sudo rm result
sudo rm debug
. ./wb55_helper_functions.sh --source-only

#main function
sudo openocd & (sleep 1 &&
(
    #echo "reset_config connect_deassert_srst"
    sleep 1
    
    execute_command "reset halt" "xpsr" "not" "not" "Unable to halt"
    echo "init"
    sleep 0.1
    if [[ $1 != *"reset"* ]]
    then
        if [[ $1 == *"program"* ]]
        then
            #Program
            execute_command "flash erase_sector 0 0 224" "erased" "error,failed" "error,failed" "Unable to erase "
            execute_command "flash write_image erase unlock $2 0x08000000" "wrote" "error" "error" "Unable to write"
        fi
        #verify memory
        execute_command "flash verify_bank 0 $2" "match" "error,differ" "error,differ" "Content from memory differs"
    fi
    #Reset and exit
    echo "reset"
    sleep 2
    if [[ error -eq 0 ]]
    then
        echo "Success" >> result
    fi
    echo "exit"
) |  telnet localhost 4444  > debug)
sudo pkill openocd


