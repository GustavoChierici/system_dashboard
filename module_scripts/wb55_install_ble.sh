#!/bin/bash

verify_error()
{
    if test -f "result"
    then
        output=$(cat result | tr -d '\0' | grep -a 'Error'| tail -1)
        if [[ $output == *"Error"* ]]
        then
            if [[ $output == *"Problem on update firmware"* ]]
            then
                echo "$1"
            else
                cat result
                exit 0
            fi
        fi
    else
        echo "Device not found"
        sudo pkill openocd
        exit 0
    fi
}

#Help
if [[ "$1" == "-h" ]]
then
  printf "Description: Install the firmwares needs to install the ble stack if necessary
  Usage: This script expect 4 arguments: firmware_ble_path, bootloader_path, [firmware_fus_older_path, firmware_fus_path], where:
   \t firmware_ble_path = path to the ble stack 
   \t bootloader_path = path to the bootloader that is used to program the firmware 
   \t firmware_fus_older_path = path to the older firmware (not required, if passed will update the fus) 
   \t firmware_fus_path = path to the current firmware 1.1.0 (required if firmware_fus_older_path, if passed will update the fus\n"
  exit 0
fi

#Warning about arguments
if [[ $# -ne 4 ]] && [[ $# -ne 2 ]]
then
    echo "firmware ble path, bootloader path, [firmware older fus path, firmware fus path] expected"
    exit 0
fi

#Verify if has ble stack
timeout 60 ./wb55_verify_if_has_ble.sh > file 2>&1
verify_error "Problem to verify ble stack install"
has_ble=$(cat result | tr -d '\0' | grep -a 'installed'| tail -1)
if [[ $has_ble == *"installed"* ]]
then
    echo "ble stack already installed"
    echo "Success"
    exit 0
fi

if [[ $# -eq 4 ]]
then
    #install older fus
    timeout 240 ./wb55_install_firmware.sh $3 $2 0 > file 2>&1
    verify_error "Problem to install older fus"

    #install current fus
    timeout 240 ./wb55_install_firmware.sh $4 $2 0 > file 2>&1
    verify_error "Problem to install current fus"    
fi

#install ble stack
timeout 240 ./wb55_install_firmware.sh $1 $2 1 > file 2>&1
verify_error "Problem to install ble"

cat result
