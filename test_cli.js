// Test Claude Code cli.js simulation
function initializeApp() {
    console.log("Starting application...");

    // Setup event handlers
    process.on("SIGINT", function() {
        console.log("Received SIGINT");
        process.exit(0);
    });

    process.on("exit", function() {
        console.log("Application exiting");
    });

    // Main render
    app.render();

    console.log("Application started");
}

// Start the app
initializeApp();
