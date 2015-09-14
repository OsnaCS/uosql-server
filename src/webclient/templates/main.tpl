<!DOCTYPE html>
<html lang="de">
<head>
    <meta accept charset="utf-8"/>
</head>
<body style = "background-color:lightgrey">
     <form>
        <button style = "text-align:right" method = "post" action = "/logout" onClick ="location = '/logout'"type="button" id = "logout"> Logout </button>
    </form>
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
    {{ result }}
</body>
