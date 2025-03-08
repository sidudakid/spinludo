document.addEventListener('DOMContentLoaded', function() {
    // Update current time
    function updateTime() {
        const now = new Date();
        const formattedTime = now.toISOString().replace('T', ' ').substring(0, 19);
        document.getElementById('current-time').textContent = formattedTime;
    }

    // Update time every second
    updateTime();
    setInterval(updateTime, 1000);
});
