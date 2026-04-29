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

function getLocation() {
    if (!navigator.geolocation) {
        alert("Geolocation is not supported by your browser.");
        return;
    }

    navigator.geolocation.getCurrentPosition(
        ({coords: { latitude, longitude }}) => {
            document.getElementById('lat').value = latitude.toFixed(6);
            document.getElementById('lon').value = longitude.toFixed(6);
        },
        (error) => {
            alert(`Unable to retrieve your location: ${error.message}`);
        },
        { enableHighAccuracy: true, timeout: 10000, maximumAge: 60000 }
    );
}

const regions = [
    { name: "Pacific Northwest", minLat: 45.5, maxLat: 49.0, minLon: -124.5, maxLon: -116.5 },
    { name: "Colorado Rockies", minLat: 37.0, maxLat: 41.0, minLon: -109.0, maxLon: -102.0 },
    { name: "Sierra Nevada", minLat: 35.0, maxLat: 40.0, minLon: -121.5, maxLon: -117.0 },
    { name: "Appalachians", minLat: 35.0, maxLat: 45.0, minLon: -84.0, maxLon: -72.0 },
];

function randomizeLocation() {
    const region = regions[Math.floor(Math.random() * regions.length)];
    const lat = region.minLat + Math.random() * (region.maxLat - region.minLat);
    const lon = region.minLon + Math.random() * (region.maxLon - region.minLon);

    document.getElementById("lat").value = lat.toFixed(6);
    document.getElementById("lon").value = lon.toFixed(6);
}
