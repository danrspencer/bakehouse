import React from 'react';
import { Container, Typography } from '@mui/material';
import type { User } from '@sample/types';

export function App() {
  const [users, setUsers] = React.useState<User[]>([]);

  React.useEffect(() => {
    fetch('http://localhost:3000/users')
      .then(res => res.json())
      .then(data => setUsers(data.data));
  }, []);

  return (
    <Container>
      <Typography variant="h1">Admin Dashboard</Typography>
      <pre>{JSON.stringify(users, null, 2)}</pre>
    </Container>
  );
} 