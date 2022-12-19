import socket
import serial
import threading
import sys

localIP     = "127.0.0.1"
localPort   = 20001
sendPort    = 20002
bufferSize  = 1024

serial_port = sys.argv[1]
ser = serial.Serial(serial_port, 115200, timeout=2)

# Create a datagram socket
UDPServerSocket = socket.socket(family=socket.AF_INET, type=socket.SOCK_DGRAM)

def read_socket():

    msgFromServer       = "Hello UDP Client"
    bytesToSend         = str.encode(msgFromServer)

    # Bind to address and ip
    UDPServerSocket.bind((localIP, localPort))
    print("UDP server up and listening" + '\r')

    # Listen for incoming datagrams
    while(True):
        bytesAddressPair = UDPServerSocket.recvfrom(bufferSize)
        message = bytesAddressPair[0]
        address = bytesAddressPair[1]

        clientMsg = "Message from Client:{}".format(message)
        clientIP  = "Client IP Address:{}".format(address)
        
        print(clientMsg + '\r')
        print(clientIP + '\r')
        
        ser.write(message)
        ser.write(b'\n\r')

def read_serial():
    while(True):
        line = ser.readline()   # read a '\n' terminated line
        if not line.strip():
            continue

        UDPServerSocket.sendto(line, (localIP, sendPort))

if __name__ == "__main__":
    readSocketThread = threading.Thread(target=read_socket)
    readSerialThread = threading.Thread(target=read_serial)

    readSocketThread.start()
    readSerialThread.start()
