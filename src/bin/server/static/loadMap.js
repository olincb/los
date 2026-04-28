function loadMap() {
    const lat = Number(document.getElementById('lat').value);
    const lon = Number(document.getElementById('lon').value);
    const container = document.getElementById('mapContainer');

    if (!Number.isFinite(lat) || lat < -90 || lat > 90) {
        container.textContent = "Latitude must be a number between -90 and 90.";
        return;
    }
    if (!Number.isFinite(lon) || lon < -180 || lon > 180) {
        container.textContent = "Longitude must be a number between -180 and 180.";
        return;
    }

    const params = new URLSearchParams({ lat: String(lat), lon: String(lon) });
    const img = document.createElement('img');
    img.alt = `Highlighted line-of-sight for lat: ${lat}, lon: ${lon}`;
    img.src = `/api/v1/highlight?${params}`;

    container.textContent = 'Loading map. This may take up to a minute.';
    img.onload = () => {
        container.replaceChildren(img);
    };
    img.onerror = () => {
        container.textContent = "Failed to load the map image. Please try again.";
    };
}
