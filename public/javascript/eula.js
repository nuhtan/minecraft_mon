function accept() {
    const response = await fetch(`${window.location.hostname}/data/accept`);
    console.log(response);
}