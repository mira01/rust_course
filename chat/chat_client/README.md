# chat_client

Terminal application for sending and receiving messages via network.

Application communicates via stdin, stdout, stderr and tcp.

Server to connect to is taken as a command line argument in form "host:port" with the default of 127.0.0.1:11111.

## Commands

* *.quit* - type .quit followed by new line to quit the application. Ctrl+C also works.
* *.image* path - Loads image from path (relative to where the app was started, or absolute) and sends to the server
* *.file* path - Load a file (simmilarly to image) and sends to the network
* any other text is sent as plain-text message. Ending a line with backshlash enables sending multiline text

# Incoming messages

* Incoming text message is displayed on stdout
* Incoming file is dowloaded into files/ folder. In case of duplicate filename, the file is overwritten
* Incoming image is attempted to convert into png and stored into images/ folder. Filename is a timestamp of download
