################################################################################
General usage
################################################################################
To send a query, enter sql-statement and confirm with return. Client-commands
are marked with a single colon in front, e.g. ":help" for access to this file.

################################################################################
Commands
################################################################################
:exit	 	Exit client without terminating server connection
:help    	Displays this file
:ping	 	Checks if server is available and active
:quit	 	Terminates connection with server and exits client

################################################################################
Startup Parameters
################################################################################
Call executable with optional Parameters to have a more refined user experience,
e.g. "uosql-server --bind=127.0.0.1 --port=4242" to connect to local host.
Inocorrect or missing parameters will result in input prompt by client.

--bind=<address>    Change the bind address.
--port=<port>       Change the port.
--name=<username>   Login with given username.
--pwd=<password>    Login with given password.
