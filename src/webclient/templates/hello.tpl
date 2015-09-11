<html>
    <body style = "background-color:lightgrey">
        <form style = "text-align:right">
            <button type="button" id = "logout"> Logout </button>
        </form>
        <h1 style = "text-align:center">
            Hello {{ name }}!
        </h1>
        <h4 style = "text-align:center; font-family:courier">
            Connected (version : {{ version }}) to {{ bind }} : {{ port }} <br>
            {{ msg }}
        </h4>

    </body>
</html>
