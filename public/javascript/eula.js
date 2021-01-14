async function accept() {
    await fetch('/api/accept');
    // refresh the page
    setTimeout(() => {
        location.reload();
    }, 2500);

}