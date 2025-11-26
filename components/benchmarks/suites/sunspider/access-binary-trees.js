// Access Binary Trees - simplified SunSpider benchmark
// Tests object creation and tree traversal

function createNode(left, right, item) {
    return {
        left: left,
        right: right,
        item: item
    };
}

function itemCheck(node) {
    if (node.left === null) {
        return node.item;
    }
    return node.item + itemCheck(node.left) - itemCheck(node.right);
}

function bottomUpTree(depth) {
    if (depth > 0) {
        return createNode(
            bottomUpTree(depth - 1),
            bottomUpTree(depth - 1),
            1
        );
    }
    return createNode(null, null, 1);
}

// Smaller depth for interpreter performance
let minDepth = 4;
let maxDepth = 6;
let stretchDepth = maxDepth + 1;

// Stretch tree
let stretchTree = bottomUpTree(stretchDepth);
let check = itemCheck(stretchTree);

// Create long-lived tree
let longLivedTree = bottomUpTree(maxDepth);

// Create and destroy trees
let iterations = 4;
for (let depth = minDepth; depth <= maxDepth; depth = depth + 2) {
    let check = 0;
    for (let i = 1; i <= iterations; i++) {
        check = check + itemCheck(bottomUpTree(depth));
    }
}

// Final check
itemCheck(longLivedTree)
