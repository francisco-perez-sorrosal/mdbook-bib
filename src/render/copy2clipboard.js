function oldCopyTextToClipboard(text) {
    var textArea = document.createElement("textarea");
    textArea.value = text;

    // Avoid scrolling to bottom
    textArea.style.top = "0";
    textArea.style.left = "0";
    textArea.style.position = "fixed";

    document.body.appendChild(textArea);
    textArea.focus();
    textArea.select();

    try {
        var ok = document.execCommand('copy');
        var msg = ok ? 'was ok' : 'failed';
        console.log('Backing copy: Text copy was ' + msg);
    } catch (err) {
        console.error('Backing copy: Unable to copy text', err);
    }

    document.body.removeChild(textArea);
}

function copyToClipboard(text) {
    if (!navigator.clipboard) {
        oldCopyTextToClipboard(text);
        return;
    }
    navigator.clipboard.writeText(text).then(function() {
        console.log('Text copied to clipboard');
    }, function(err) {
        console.error('Error copying text: ', err);
    });
}
