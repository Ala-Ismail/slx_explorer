const carousel = document.querySelector('[data-carousel]');
const slides = [...carousel.querySelectorAll('[data-slide]')];
const controls = carousel.querySelector('.carousel-controls');
const rotation = carousel.querySelector('[data-rotation]');
const count = carousel.querySelector('[data-count]');
const announcement = carousel.querySelector('[data-announcement]');
const reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)');

let current = 0;
let rotating = !reducedMotion.matches;
let hovering = false;
let timer;

function showSlide(index, announce = false) {
  slides[current].hidden = true;
  current = (index + slides.length) % slides.length;
  const slide = slides[current];
  slide.hidden = false;
  count.textContent = `${current + 1} / ${slides.length}`;
  if (announce) {
    const caption = slide.querySelector('figcaption').textContent;
    announcement.textContent = `Slide ${current + 1} of ${slides.length}. ${caption}`;
  }
}

function updateRotation() {
  window.clearTimeout(timer);
  rotation.textContent = rotating ? 'Pause' : 'Play';
  rotation.setAttribute('aria-label', rotating ? 'Pause slideshow' : 'Play slideshow');
  if (rotating && !hovering && !document.hidden) {
    timer = window.setTimeout(() => {
      showSlide(current + 1);
      updateRotation();
    }, 6500);
  }
}

function moveSlide(step) {
  rotating = false;
  showSlide(current + step, true);
  updateRotation();
}

carousel.querySelector('[data-previous]').addEventListener('click', () => moveSlide(-1));
carousel.querySelector('[data-next]').addEventListener('click', () => moveSlide(1));
rotation.addEventListener('click', () => {
  rotating = !rotating;
  updateRotation();
});
carousel.addEventListener('pointerenter', event => {
  hovering = event.pointerType === 'mouse';
  updateRotation();
});
carousel.addEventListener('pointerleave', () => {
  hovering = false;
  updateRotation();
});
carousel.addEventListener('focusin', event => {
  if (event.target !== rotation) {
    rotating = false;
    updateRotation();
  }
});
reducedMotion.addEventListener('change', event => {
  if (event.matches) {
    rotating = false;
    updateRotation();
  }
});
document.addEventListener('visibilitychange', updateRotation);

controls.hidden = false;
updateRotation();
