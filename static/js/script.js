async function createGame() {
    const entryFee = document.getElementById('entry-fee').value;
    const ownerCut = document.getElementById('owner-cut').value;

    const response = await fetch('/api/games', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify({
            entry_fee: entryFee,
            owner_cut: ownerCut
        })
    });

    const message = await response.json();
    showMessage('Create Game: ' + JSON.stringify(message));
}

async function joinGame() {
    const gameId = document.getElementById('game-id').value;
    const player2Id = document.getElementById('player2-id').value;

    const response = await fetch(`/api/games/${gameId}/join`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify({
            player2_id: player2Id
        })
    });

    const message = await response.json();
    showMessage('Join Game: ' + JSON.stringify(message));
}

async function startGame() {
    const gameId = document.getElementById('action-game-id').value;

    const response = await fetch(`/api/games/${gameId}/start`, {
        method: 'POST'
    });

    const message = await response.json();
    showMessage('Start Game: ' + JSON.stringify(message));
}

async function endGame() {
    const gameId = document.getElementById('action-game-id').value;
    const winnerId = document.getElementById('winner-id').value;

    const response = await fetch(`/api/games/${gameId}/end`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify({
            winner_id: winnerId
        })
    });

    const message = await response.json();
    showMessage('End Game: ' + JSON.stringify(message));
}

function showMessage(message) {
    const messageDiv = document.getElementById('messages');
    messageDiv.innerText = message;
    messageDiv.style.display = 'block';
}