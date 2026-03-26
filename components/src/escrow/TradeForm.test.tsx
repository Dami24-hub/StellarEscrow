import React from 'react';
import { render, screen, fireEvent, waitFor, act } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { TradeForm } from './TradeForm';

// Suppress localStorage errors in jsdom
beforeEach(() => {
  localStorage.clear();
  jest.useFakeTimers();
});

afterEach(() => {
  jest.runOnlyPendingTimers();
  jest.useRealTimers();
});

const VALID_SELLER = 'GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWN';
const VALID_BUYER  = 'GBVVJJWT3JTHKBZISFYYQDKDS3HHGBYPSLFGGT27CDMBC3JTHKBZISFYY';

describe('TradeForm — rendering', () => {
  it('renders all form fields', () => {
    render(<TradeForm onSubmit={jest.fn()} disableAutoSave />);
    expect(screen.getByLabelText('Seller Address')).toBeInTheDocument();
    expect(screen.getByLabelText('Buyer Address')).toBeInTheDocument();
    expect(screen.getByLabelText('Amount (USDC)')).toBeInTheDocument();
    expect(screen.getByLabelText('Arbitrator Address (Optional)')).toBeInTheDocument();
  });

  it('disables submit button when loading', () => {
    render(<TradeForm onSubmit={jest.fn()} loading disableAutoSave />);
    expect(screen.getByRole('button', { name: /create trade/i })).toBeDisabled();
  });
});

describe('TradeForm — submit-time validation', () => {
  it('shows all required errors on empty submit', () => {
    render(<TradeForm onSubmit={jest.fn()} disableAutoSave />);
    fireEvent.click(screen.getByRole('button', { name: /create trade/i }));

    expect(screen.getByText('Seller address is required')).toBeInTheDocument();
    expect(screen.getByText('Buyer address is required')).toBeInTheDocument();
    expect(screen.getByText('Amount is required')).toBeInTheDocument();
  });

  it('does not call onSubmit when validation fails', () => {
    const onSubmit = jest.fn();
    render(<TradeForm onSubmit={onSubmit} disableAutoSave />);
    fireEvent.click(screen.getByRole('button', { name: /create trade/i }));
    expect(onSubmit).not.toHaveBeenCalled();
  });

  it('calls onSubmit with valid data', () => {
    const onSubmit = jest.fn();
    render(<TradeForm onSubmit={onSubmit} disableAutoSave />);

    fireEvent.change(screen.getByLabelText('Seller Address'), { target: { value: VALID_SELLER } });
    fireEvent.change(screen.getByLabelText('Buyer Address'),  { target: { value: VALID_BUYER } });
    fireEvent.change(screen.getByLabelText('Amount (USDC)'),  { target: { value: '100' } });

    fireEvent.click(screen.getByRole('button', { name: /create trade/i }));

    expect(onSubmit).toHaveBeenCalledWith({
      seller: VALID_SELLER,
      buyer: VALID_BUYER,
      amount: '100',
      arbitrator: '',
    });
  });
});

describe('TradeForm — Stellar address validation', () => {
  it('shows address format error for invalid seller', () => {
    render(<TradeForm onSubmit={jest.fn()} disableAutoSave />);
    fireEvent.change(screen.getByLabelText('Seller Address'), { target: { value: 'G123' } });
    fireEvent.blur(screen.getByLabelText('Seller Address'));
    expect(screen.getByText(/valid Stellar address/i)).toBeInTheDocument();
  });

  it('shows error when buyer equals seller', () => {
    render(<TradeForm onSubmit={jest.fn()} disableAutoSave />);
    fireEvent.change(screen.getByLabelText('Seller Address'), { target: { value: VALID_SELLER } });
    fireEvent.change(screen.getByLabelText('Buyer Address'),  { target: { value: VALID_SELLER } });
    fireEvent.blur(screen.getByLabelText('Buyer Address'));
    expect(screen.getByText(/buyer and seller must be different/i)).toBeInTheDocument();
  });

  it('accepts a valid arbitrator address', () => {
    const onSubmit = jest.fn();
    render(<TradeForm onSubmit={onSubmit} disableAutoSave />);

    fireEvent.change(screen.getByLabelText('Seller Address'),              { target: { value: VALID_SELLER } });
    fireEvent.change(screen.getByLabelText('Buyer Address'),               { target: { value: VALID_BUYER } });
    fireEvent.change(screen.getByLabelText('Amount (USDC)'),               { target: { value: '50' } });
    fireEvent.change(screen.getByLabelText('Arbitrator Address (Optional)'), { target: { value: VALID_SELLER } });

    fireEvent.click(screen.getByRole('button', { name: /create trade/i }));
    expect(onSubmit).toHaveBeenCalled();
  });

  it('rejects an invalid arbitrator address', () => {
    render(<TradeForm onSubmit={jest.fn()} disableAutoSave />);
    fireEvent.change(screen.getByLabelText('Arbitrator Address (Optional)'), { target: { value: 'bad' } });
    fireEvent.blur(screen.getByLabelText('Arbitrator Address (Optional)'));
    expect(screen.getByText(/valid Stellar address/i)).toBeInTheDocument();
  });
});

describe('TradeForm — real-time validation', () => {
  it('does not show errors before a field is touched', () => {
    render(<TradeForm onSubmit={jest.fn()} disableAutoSave />);
    // No interaction — no errors should be visible
    expect(screen.queryByRole('alert')).not.toBeInTheDocument();
  });

  it('shows error immediately after blur on empty required field', () => {
    render(<TradeForm onSubmit={jest.fn()} disableAutoSave />);
    fireEvent.blur(screen.getByLabelText('Seller Address'));
    expect(screen.getByText('Seller address is required')).toBeInTheDocument();
  });

  it('clears error when field becomes valid after blur', () => {
    render(<TradeForm onSubmit={jest.fn()} disableAutoSave />);
    const input = screen.getByLabelText('Seller Address');

    fireEvent.blur(input);
    expect(screen.getByText('Seller address is required')).toBeInTheDocument();

    fireEvent.change(input, { target: { value: VALID_SELLER } });
    expect(screen.queryByText('Seller address is required')).not.toBeInTheDocument();
  });

  it('shows amount error for non-numeric value', () => {
    render(<TradeForm onSubmit={jest.fn()} disableAutoSave />);
    const input = screen.getByLabelText('Amount (USDC)');
    fireEvent.change(input, { target: { value: 'abc' } });
    fireEvent.blur(input);
    expect(screen.getByText(/positive number/i)).toBeInTheDocument();
  });

  it('shows amount error for zero', () => {
    render(<TradeForm onSubmit={jest.fn()} disableAutoSave />);
    const input = screen.getByLabelText('Amount (USDC)');
    fireEvent.change(input, { target: { value: '0' } });
    fireEvent.blur(input);
    expect(screen.getByText(/positive number/i)).toBeInTheDocument();
  });
});

describe('TradeForm — auto-save', () => {
  it('saves draft to localStorage after debounce', async () => {
    render(<TradeForm onSubmit={jest.fn()} />);

    fireEvent.change(screen.getByLabelText('Seller Address'), { target: { value: VALID_SELLER } });

    act(() => { jest.advanceTimersByTime(900); });

    expect(localStorage.getItem('stellar_escrow_trade_form_draft')).not.toBeNull();
    const draft = JSON.parse(localStorage.getItem('stellar_escrow_trade_form_draft')!);
    expect(draft.seller).toBe(VALID_SELLER);
  });

  it('restores draft from localStorage on mount', () => {
    localStorage.setItem(
      'stellar_escrow_trade_form_draft',
      JSON.stringify({ seller: VALID_SELLER, buyer: '', amount: '42', arbitrator: '' })
    );

    render(<TradeForm onSubmit={jest.fn()} />);

    expect((screen.getByLabelText('Seller Address') as HTMLInputElement).value).toBe(VALID_SELLER);
    expect((screen.getByLabelText('Amount (USDC)') as HTMLInputElement).value).toBe('42');
  });

  it('clears draft on successful submit', () => {
    localStorage.setItem(
      'stellar_escrow_trade_form_draft',
      JSON.stringify({ seller: VALID_SELLER, buyer: VALID_BUYER, amount: '10', arbitrator: '' })
    );

    render(<TradeForm onSubmit={jest.fn()} />);

    fireEvent.change(screen.getByLabelText('Seller Address'), { target: { value: VALID_SELLER } });
    fireEvent.change(screen.getByLabelText('Buyer Address'),  { target: { value: VALID_BUYER } });
    fireEvent.change(screen.getByLabelText('Amount (USDC)'),  { target: { value: '10' } });
    fireEvent.click(screen.getByRole('button', { name: /create trade/i }));

    expect(localStorage.getItem('stellar_escrow_trade_form_draft')).toBeNull();
  });

  it('does not save when disableAutoSave is true', () => {
    render(<TradeForm onSubmit={jest.fn()} disableAutoSave />);
    fireEvent.change(screen.getByLabelText('Seller Address'), { target: { value: VALID_SELLER } });
    act(() => { jest.advanceTimersByTime(1000); });
    expect(localStorage.getItem('stellar_escrow_trade_form_draft')).toBeNull();
  });
});
