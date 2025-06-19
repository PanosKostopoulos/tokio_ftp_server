temporary solution to case file ...


the File sent ok message which works like an ACK sometimes it is read with the last chunk of the file and sometimes doesn't. this is bad so in the future we should have seperate control plane and data plane



after a file has been sent client should calculate a checksum and also the ftp server so they see if they match

i could add more functionalities such as mkdir upload files renames moves etc....
maybe implement continuing of download?

implement standardised commands
https://en.wikipedia.org/wiki/List_of_FTP_commands
https://www.rfc-editor.org/rfc/rfc959.html
