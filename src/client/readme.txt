
################################################################################
General usage
################################################################################
To send a query, enter sql-statement and confirm with return. Client-commands
are marked with a single colon in front, e.g. ":help" for access to this file.

################################################################################
Commands
################################################################################
:exit	 	Exit client without terminating server connection.
:help    	Displays this file.
:ping	 	Checks if server is available and active.
:quit	 	Terminates connection with server and exits client.

################################################################################
Startup parameters
################################################################################
Call executable with optional Parameters to have a more refined user experience,
e.g. "uosql-server --bind=192.168.1.59 --port=30" to connect to local host.
Parameters can be applied in any order. Inocorrect or missing parameters will
result in input prompt by client for the specific values. If no further input
is provided by pressing "return", default values will be used to establish a
connection to local host (127.0.0.1:4242).

--bind=<address>    Change the bind address.
--port=<port>       Change the port.
--name=<username>   Login with given username.
--pwd=<password>    Login with given password.
