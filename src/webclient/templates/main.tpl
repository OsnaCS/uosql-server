<!DOCTYPE html>
<html lang="de">
<head>
    <meta accept charset="utf-8"/>
    <style>
        table, th, td {
            border: 1px solid black;
            border-collapse: collapse;
        }
        td, th {
            padding: 5px;
            text-align: left;
        }
        table#t01 {
            background-color: #ffffff;
            width: 70%;
            margin-left: 15%;
            margin-right: 15%;
        }
    </style>
</head>
<body style = "background-color:#ffffff">
    <a href="http://www.uni-osnabrueck.de">
        <img src="http://2011.bewegtekindheit.de/pics/pics_master/unilogo.jpg" style="width:20%;height:20%">
    </a>
    <h1 style = "text-align:center">
        Hello {{ name }}!
    </h1>
    <h4 style = "text-align:center; font-family:courier">
        Connected (version : {{ version }}) to {{ bind }} : {{ port }} <br>
        {{ msg }}
    </h4>
    <form style="text-align:center">
        <textarea name="sql" rows="5" cols="50"></textarea><br>
        <input type = "submit" value="Query">
    </form>
    <p style = "text-align:center">
        <pre>
            <font face="Verdana" size="3">
                {{{ result }}}
            </font>
        </pre>
    </p>
    <form style = "text-align:right">
        <button method = "post" action = "/logout" onClick ="location = '/logout'"type="button" id = "logout"> Logout </button>
    </form>
    <p style="text-align:right">
        <a href="http://media2mult.uni-osnabrueck.de/pmwiki/fields/dbp15/">
            Project/Code Documentation
        </a>
    </p>
</body>
