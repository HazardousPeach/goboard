width = 512;
height = 480;

piece_width = width * 0.05;
piece_height = height * 0.05;

tile_width = width * 0.052;
tile_height = height * 0.052;

board_offset_x = 2;
board_offset_y = 2;

board_size = 19;

players_turn = true;

var GoImage = function (src) {
    this.image = new Image();
    this.image.src = src;
    this.ready = false;
    this.image.onload = function () {
        this.ready = true;
        render();
    }
}
GoImage.prototype.maybeDraw = function (x, y, width, height) {
    ctx.drawImage(this.image, x, y, width, height);
}

window.onload = function () {
    var makeMove = function(e) {
        board_pos = [Math.round((e.clientX / tile_width)) - 1,
                     Math.round((e.clientY / tile_height)) - 1];
        if (hasPiece(board_pos))
            alert("There's already a piece there!");
        else if (!players_turn)
            alert("It's not your turn!");
        else {
            whitePieces.push(board_pos);
            handle_captures();
            render();
            players_turn = false;
            socket.send(board_pos[0] + "," + board_pos[1]);
        }
    }

    var receive_state = function(e) {
        var board_string = e.data;
        whitePieces = [];
        blackPieces = [];
        for(i = 0; i < board_size; i++){
            for (j = 0; j < board_size; j++){
                tile = board_string[j * (board_size + 1) + i];
                if( tile == 'W')
                    whitePieces.push([i,j]);
                else if (tile == 'B')
                    blackPieces.push([i,j]);
            }
        }
        players_turn = true;
        render();
    }

    render = function () {
        board.maybeDraw(0,0, width, height);
        for (i = 0; i < whitePieces.length; i++)
            whitePiece.maybeDraw(whitePieces[i][0] * tile_width + (tile_width - piece_width) / 2 + board_offset_x,
                                 whitePieces[i][1] * tile_height + (tile_height - piece_height) / 2 + board_offset_y,
                                 piece_width, piece_height);
        for (i = 0; i < blackPieces.length; i++)
            blackPiece.maybeDraw(blackPieces[i][0] * tile_width + (tile_width - piece_width) / 2 + board_offset_x,
                                 blackPieces[i][1] * tile_height + (tile_height - piece_height) / 2 + board_offset_y,
                                 piece_width, piece_height);
    };

    var hasPiece = function (pos) {
        for (i = 0; i < whitePieces.length; i++){
            if (pos[0] == whitePieces[i][0] &&
                pos[1] == whitePieces[i][1])
                return true;
        }
        for (i = 0; i < blackPieces.length; i++){
            if (pos[0] == blackPieces[i][0] &&
                pos[1] == blackPieces[i][1])
                return true;
        }
        return false;
    }

    var httpGet = function(url, callback) {
        xmlhttp=new XMLHttpRequest();
        xmlhttp.onreadystatechange = function(){
            if (xmlhttp.readyState == 4 && xmlhttp.status == 200)
                callback(xmlhttp.responseText);
        }
        xmlhttp.open("GET", url, false);
        xmlhttp.send();
    }

    var connectToServer = function(ip) {
        socket = new WebSocket("ws:" + ip + ":2794", "rust-websocket");
        socket.onopen = function (event) {
            // Initialize input on the game board.
            canvas.addEventListener("click", makeMove);
        }
        socket.onmessage = receive_state;
    }

    var handle_captures = function () {
    }

    // Create the canvas
    var canvas = document.createElement("canvas");
    ctx = canvas.getContext("2d");
    canvas.width = width;
    canvas.height = height;
    document.body.appendChild(canvas);

    // Load the board
    var board = new GoImage("images/board.png");

    // Get the pieces ready
    var whitePieces = [];
    var whitePiece = new GoImage("images/white_piece.png");

    var blackPieces = [];
    var blackPiece = new GoImage("images/black_piece.png");

    // Connect to the server
    httpGet("http://alex.uwplse.org/homeserver_ip.txt", connectToServer)
}

