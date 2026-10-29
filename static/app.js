(() => {
  document.querySelectorAll('[data-table-search]').forEach(input => {
    input.addEventListener('input', () => {
      const q = input.value.normalize('NFD').replace(/[\u0300-\u036f]/g, '').toLowerCase();
      document.querySelectorAll('[data-searchable] tbody tr').forEach(row => {
        row.hidden = !row.textContent.normalize('NFD').replace(/[\u0300-\u036f]/g, '').toLowerCase().includes(q);
      });
    });
  });
  const form = document.getElementById('checkout-form');
  if (form) {
    const family = document.getElementById('checkout-family');
    const familySearch = document.getElementById('checkout-family-search');
    const familyCount = document.getElementById('checkout-family-count');
    const familyOptions = [...family.options].slice(1).map(option => option.cloneNode(true));
    const quantities = [...form.querySelectorAll('[data-price]')];
    const format = n => (n / 100).toLocaleString('pt-BR', {minimumFractionDigits: 2, maximumFractionDigits: 2});
    const normalize = value => value.normalize('NFD').replace(/[\u0300-\u036f]/g, '').toLowerCase();
    const update = () => {
      const total = quantities.reduce((sum, el) => sum + (Number(el.value) || 0) * Number(el.dataset.price), 0);
      const credit = Number(family.selectedOptions[0]?.dataset.credit || 0);
      document.getElementById('cart-total').textContent = format(total) + ' créditos';
      const balance = document.getElementById('cart-balance');
      balance.textContent = family.value ? `Crédito mensal: ${format(credit)} · Saldo após a entrega: ${format(credit - total)}` : 'Escolha uma família para ver o saldo.';
      balance.classList.toggle('negative', total > credit);
    };
    familySearch.addEventListener('input', () => {
      const query = normalize(familySearch.value.trim());
      const selected = family.value;
      const matches = familyOptions.filter(option => normalize(option.textContent).includes(query));
      const placeholder = document.createElement('option');
      placeholder.value = '';
      placeholder.disabled = true;
      placeholder.selected = true;
      placeholder.textContent = matches.length ? 'Selecione uma família elegível…' : 'Nenhuma família corresponde à busca';
      family.replaceChildren(placeholder, ...matches);
      if (matches.some(option => option.value === selected)) family.value = selected;
      familyCount.textContent = query
        ? `${matches.length} de ${familyOptions.length} famílias elegíveis correspondem à busca.`
        : `${familyOptions.length} famílias elegíveis em ${familyCount.dataset.month}. Famílias atendidas ou fora da vez não aparecem nesta lista.`;
      update();
    });
    form.addEventListener('input', update);
    family.addEventListener('change', () => {
      if (family.value) familySearch.value = family.selectedOptions[0].textContent;
      update();
    });
  }
  document.querySelectorAll('[data-print]').forEach(button => button.addEventListener('click', () => window.print()));
  document.querySelectorAll('form[method="post"]').forEach(form => {
    form.addEventListener('submit', event => {
      const button = event.submitter;
      if (button?.dataset.confirm && !window.confirm(button.dataset.confirm)) { event.preventDefault(); return; }
      if (form.dataset.submitted) { event.preventDefault(); return; }
      form.dataset.submitted = 'true';
      if (button) { button.disabled = true; button.textContent = 'Salvando…'; }
    });
  });
  window.addEventListener('pageshow', event => { if (event.persisted) location.reload(); });
})();
