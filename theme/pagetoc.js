// Un-active everything when you click it
Array.prototype.forEach.call(document.getElementsByClassName("pagetoc")[0].children, function(el) {
    el.addEventHandler("click", function() {
        Array.prototype.forEach.call(document.getElementsByClassName("pagetoc")[0].children, function(el) {
            el.classList.remove("active");
        });
        el.classList.add("active");
    });
});

var updateFunction = function() {

    var id;
    var elements = document.getElementsByClassName("header");
    Array.prototype.forEach.call(elements, function(el) {
        if (window.pageYOffset >= el.offsetTop) {
            id = el;
        }
    });

    Array.prototype.forEach.call(document.getElementsByClassName("pagetoc")[0].children, function(el) {
        el.classList.remove("active");
    });

    Array.prototype.forEach.call(document.getElementsByClassName("pagetoc")[0].children, function(el) {
        if (id.href.localeCompare(el.href) == 0) {
            el.classList.add("active");
        }
    });
};

const indentLevels = {
    H1: 0,
    H2: 1,
    H3: 2,
    H4: 3,
    H5: 4,
    H6: 5,
}

// Populate sidebar on load
window.addEventListener('load', function() {
    var pagetoc = document.getElementsByClassName("pagetoc")[0];
    var elements = document.getElementsByClassName("header");
    Array.prototype.forEach.call(elements, function(el) {
        const link = document.createElement("a");
        const indent = indentLevels[el.parentElement.tagName]

        if (indent) {
            link.style.paddingLeft = `${indent * 20}px`;
        }

        link.appendChild(document.createTextNode(el.text));
        link.href = el.href;
        pagetoc.appendChild(link);
    });
    updateFunction.call();
});



// Handle active elements on scroll
window.addEventListener("scroll", updateFunction);
