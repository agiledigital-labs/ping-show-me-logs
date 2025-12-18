type LogProps = { data: Record<string, unknown> };

export const Log = ({ data }: LogProps) => (
  <>
    <p>
      <b>{String(data.level)}</b>{" "}
      {/** @ts-expect-error -- Can't be bothered. */}
      {String(data.message ?? JSON.stringify(data.entries?.[0]?.info))}
    </p>
  </>
);
