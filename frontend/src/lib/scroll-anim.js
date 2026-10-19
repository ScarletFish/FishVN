const SCROLLER = '.shell__main';

export function createScrollAnim() {
  const state = {
    sectionTriggers: [],
    cardTriggers: [],
    sectionEls: [],
    cardEls: [],
  };

  function clearInline(el) {
    el.style.opacity = '';
    el.style.visibility = '';
    el.style.transform = '';
  }

  function kill() {
    state.sectionTriggers.forEach((t) => t && t.kill());
    state.cardTriggers.forEach((t) => t && t.kill());
    for (const el of state.sectionEls) clearInline(el);
    for (const el of state.cardEls) clearInline(el);
    state.sectionTriggers = [];
    state.cardTriggers = [];
    state.sectionEls = [];
    state.cardEls = [];
  }

  function build({ sections = [], cards = [] } = {}) {
    kill();
    const gsap = globalThis.gsap;
    if (!gsap || !gsap.ScrollTrigger) return;
    if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) return;
    const scroller = document.querySelector(SCROLLER);
    if (!scroller) return;

    for (const el of sections) {
      const tween = gsap.fromTo(
        el,
        { autoAlpha: 0, y: 16 },
        {
          autoAlpha: 1,
          y: 0,
          duration: 0.5,
          ease: 'power2.out',
          scrollTrigger: { trigger: el, start: 'top 92%', once: true, scroller },
        },
      );
      if (tween.scrollTrigger) {
        state.sectionTriggers.push(tween.scrollTrigger);
        state.sectionEls.push(el);
      }
    }

    const cardList = Array.from(cards);
    if (cardList.length) {
      gsap.set(cardList, { autoAlpha: 0, y: 24 });
      state.cardEls.push(...cardList);
      state.cardTriggers.push(
        ...gsap.ScrollTrigger.batch(cardList, {
          start: 'top 92%',
          once: true,
          scroller,
          onEnter: (batch) =>
            gsap.to(batch, {
              y: 0,
              autoAlpha: 1,
              duration: 0.5,
              ease: 'power2.out',
              stagger: 0.04,
            }),
        }),
      );
    }
  }

  return { build, kill };
}
