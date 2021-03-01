# rats-ticket-service
Simple service where tickets can be created and approved

## Install
To install download the release, extract it and run `sudo ./install.sh` to install the necessary
files.

Run `systemctl enable rats.service --now` to enable and start the server.

A .doc-File can be placed at `/opt/rats/uploads/template.doc` to be available as a downloadable
template for ticket creation.

For correct sending of emails set the environment variables `ADMIN_EMAIL` (email-address of admin
who gets notified if something is approved), `GMAIL_USERNAME` and `GMAIL_PASSWORD` (for the email
account which is used to send the emails.
