
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
:load       Loads script.sql from client-folder and executes querys. See
            script.sql for further information on syntax. 
:ping	 	Checks if server is available and active.
:quit	 	Terminates connection with server and exits client.

################################################################################
Startup parameters
################################################################################
Call executable with optional parameters to have a more refined user experience,
e.g. "uosql-server --bind=192.168.1.59 --port=30" to automatically connect on
startup to the given ip at the specified port. Parameters can be applied in any
order. Incorrect or missing parameters will result in input prompt by client
in order to obtain the needed information. If no further input is provided by
pressing "return", default values will be used to establish a connection.
Default IP is 127.0.0.1, default Port is 4242

--bind=<address>    Change the bind address. Standard format is "w.x.y.z" with
                    w,x,y,z consisting of one up to three digits.
--port=<port>       Change the port. Has to be numeric value.
--name=<username>   Login with given username.
--pwd=<password>    Login with given password.
