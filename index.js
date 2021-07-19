const configureSidebar = () => {
  const sidebar = document.querySelector(".sidebar");

  document
    .querySelector(".sidebar-button")
    ?.addEventListener("click", function () {
      this.classList.toggle("on");
      sidebar.classList.toggle("hidden");
    });
};

const configurePrerenderHooks = () => {
  const link = document.querySelector('link[rel="prerender"]');

  for (const element of Array.from(document.getElementsByTagName("a"))) {
    element.addEventListener("mouseover", function () {
      link.setAttribute("href", this.getAttribute("href"));
    });
  }
};

configureSidebar();
configurePrerenderHooks();
