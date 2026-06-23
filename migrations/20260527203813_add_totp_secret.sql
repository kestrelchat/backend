ALTER TABLE public.accounts
ADD COLUMN totp_secret TEXT DEFAULT NULL;
