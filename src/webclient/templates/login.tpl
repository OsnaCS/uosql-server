<!DOCTYPE html>
<html lang="de">
<head>
    <meta accept charset="utf-8"/>
</head>
<body style = "background-color:#ffffff">
    <a href="http://www.uni-osnabrueck.de">
        <img src="http://2011.bewegtekindheit.de/pics/pics_master/unilogo.jpg" style="width:20%;height:20%">
    </a>
    <h1 style = "text-align:center">
        Login
    </h1>
    <p style="text-align:center;color:red">
        {{ err_msg }}
    </p>
    <br>
    <p>
        <form style = "text-align:center" method = "post" action="/login">
            <label for ="user"> Username*<br></label>
            <input type = "text" name="user" id="user" required><br>
            <label for ="password"> Password*<br></label>
            <input type = "password" name="password" id="password" required><br>
            <label for ="Bind"> Bind to<br></label>
            <input type = "text" name="bind" id="bind"><br>
            <label for ="port"> Port<br></label>
            <input type = "port" name="port" id="port"><br>
            <input type = "submit" value="Login">
        </form>
    </p>
    <h6 style = "text-align:center">
        *required
    <h6>
    <p style="text-align:right">
        <a href="http://media2mult.uni-osnabrueck.de/pmwiki/fields/dbp15/">
            Project/Code Documentation
        </a>
    </p>
</body>
