// 3D Cube Rotation - simplified SunSpider benchmark
// Tests floating point math and loops

let vertices = [
    [-1, -1, -1],
    [ 1, -1, -1],
    [ 1,  1, -1],
    [-1,  1, -1],
    [-1, -1,  1],
    [ 1, -1,  1],
    [ 1,  1,  1],
    [-1,  1,  1]
];

function rotateX(point, angle) {
    let y = point[1];
    let z = point[2];
    point[1] = y * 0.866 - z * 0.5;  // cos(30°) ≈ 0.866, sin(30°) = 0.5
    point[2] = y * 0.5 + z * 0.866;
}

function rotateY(point, angle) {
    let x = point[0];
    let z = point[2];
    point[0] = x * 0.866 + z * 0.5;
    point[2] = -x * 0.5 + z * 0.866;
}

function rotateZ(point, angle) {
    let x = point[0];
    let y = point[1];
    point[0] = x * 0.866 - y * 0.5;
    point[1] = x * 0.5 + y * 0.866;
}

let iterations = 1000;
for (let i = 0; i < iterations; i++) {
    for (let j = 0; j < vertices.length; j++) {
        rotateX(vertices[j], 0.1);
        rotateY(vertices[j], 0.1);
        rotateZ(vertices[j], 0.1);
    }
}

// Return a checksum
let checksum = 0;
for (let i = 0; i < vertices.length; i++) {
    checksum = checksum + vertices[i][0] + vertices[i][1] + vertices[i][2];
}
checksum
