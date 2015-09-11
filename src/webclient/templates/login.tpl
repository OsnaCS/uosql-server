<html>
    <body style = "background-color: lightgrey">
        <h1 style = "text-align:center">
            Login
        </h1>
        <p style="text-align:center">
            {{ err_msg }}
        </p>
        <p>
            <form style = "text-align:center" method = "post" action="/login">
                <label for ="user"> Username:</label>
                <input type = "text" name="user" id="user" required><br>
                <label for ="password"> Password:</label>
                <input type = "password" name="password" id="password" required><br>
                <input type = "submit" value="Login">
            </form>
        </p>
    </body>
</html>
