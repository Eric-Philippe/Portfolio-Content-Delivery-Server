-- Sample data to test the server

-- Development projects
INSERT INTO DevProjectMetadata (slug, en_title, en_short_description, fr_title, fr_short_description, techs, link, date, tags) VALUES 
('portfolio-server', 'Portfolio Server', 'A lightweight Rust server for portfolio content', 'Serveur Portfolio', 'Un serveur Rust léger pour le contenu de portfolio', 'Rust,Axum,SQLite', 'https://github.com/username/portfolio-server', '2025-06-13', 'web,backend,api'),
('photo-gallery', 'Photo Gallery App', 'Modern photo gallery with responsive design', 'Application Galerie Photo', 'Galerie photo moderne avec design responsive', 'React,TypeScript,Tailwind', 'https://github.com/username/photo-gallery', '2025-05-20', 'frontend,react,photography');

-- Photo albums
INSERT INTO AlbumMetadata (slug, title, description, short_title, date, camera, lens, phone, preview_img_one_url, feature, category) VALUES 
('urban-exploration', 'Urban Exploration 2025', 'Exploring the city through photography', 'Urban 2025', '2025-06-01', 'Canon EOS R5', 'RF 24-70mm f/2.8L', NULL, '/files/urban-exploration/preview1.jpg', true, 'Street'),
('nature-walks', 'Nature Walks', 'Peaceful moments in nature', 'Nature', '2025-05-15', NULL, NULL, 'iPhone 15 Pro', '/files/nature-walks/preview1.jpg', false, 'Nature');

-- Album content (adapt according to your actual images)
INSERT INTO AlbumContent (slug, img_url, caption, img_path) VALUES 
('urban-exploration', '/files/urban-exploration/street1.jpg', 'Street art in downtown', 'uploads/urban-exploration/street1.jpg'),
('urban-exploration', '/files/urban-exploration/building1.jpg', 'Modern architecture', 'uploads/urban-exploration/building1.jpg'),
('nature-walks', '/files/nature-walks/forest1.jpg', 'Morning light through trees', 'uploads/nature-walks/forest1.jpg');
