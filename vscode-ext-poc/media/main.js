//@ts-check

// This script will be run within the webview itself
// It cannot access the main VS Code APIs directly.
(function () {
	const vscode = acquireVsCodeApi();
	window.addEventListener(
		'messageerror',
		(event) => {
			console.log(event);
		},
		false
	);
	window.addEventListener(
		'message',
		(event) => {
			console.log(event);
			vscode.postMessage(event.data)
		},
		false
	);

	console.error("the fn was init'ed");
})();
