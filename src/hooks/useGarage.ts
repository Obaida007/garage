import { useGarageStore } from "@/stores/garageStore";

export function useGarage() {
  const snapshot = useGarageStore((s) => s.snapshot);
  const locale = useGarageStore((s) => s.locale);
  const loading = useGarageStore((s) => s.loading);
  const error = useGarageStore((s) => s.error);
  const refresh = useGarageStore((s) => s.refresh);
  return { snapshot, locale, loading, error, refresh };
}
